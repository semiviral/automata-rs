use std::{
    collections::LinkedList,
    lazy::OnceCell,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

pub struct MemorySlice<'a, T> {
    pool: &'a MemoryPool,
    index: usize,
    slice: &'a mut [T],
}

impl<T> Drop for MemorySlice<'_, T> {
    fn drop(&mut self) {
        self.pool.return_slice(self)
    }
}

struct MemoryBlock {
    index: usize,
    size: usize,
    owned: bool,
}

pub struct MemoryPool {
    head: *mut u8,
    map: Mutex<LinkedList<MemoryBlock>>,
    total_bytes: AtomicUsize,
    rented_bytes: AtomicUsize,
    rented_blocks: AtomicUsize,
}

impl<T> core::ops::Index<usize> for MemorySlice<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slice[index]
    }
}

impl<T> core::ops::IndexMut<usize> for MemorySlice<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slice[index]
    }
}

impl MemoryPool {
    pub fn new(head: *mut u8, byte_len: usize) -> Self {
        let mut map = LinkedList::new();
        map.push_front(MemoryBlock {
            index: 0,
            size: byte_len,
            owned: false,
        });

        Self {
            head,
            map: Mutex::new(map),
            total_bytes: AtomicUsize::new(byte_len),
            rented_bytes: AtomicUsize::new(0),
            rented_blocks: AtomicUsize::new(0),
        }
    }

    pub fn remaining_bytes(&self) -> usize {
        self.total_bytes.load(Ordering::Acquire) - self.rented_bytes.load(Ordering::Acquire)
    }

    pub fn rent_slice<'a, T>(
        &'a self,
        size: NonZeroUsize,
        alignment: NonZeroUsize,
        zero_memory: bool,
    ) -> Option<MemorySlice<'a, T>> {
        let mut map = self
            .map
            .lock()
            .expect("Memory pool map mutex has been poisoned!");

        // Calculate the real size in bytes of a rental request.
        let size_in_bytes = size.get() * std::mem::size_of::<T>();

        // If the pool cannot serve a request, fail early.
        if size_in_bytes > self.remaining_bytes() {
            return None;
        }

        let mut map_cursor = map.cursor_front_mut();
        let mut before_cursor = OnceCell::new();
        let mut after_cursor = OnceCell::new();
        let mut memory_slice = OnceCell::new();

        while let Some(block) = map_cursor.current() {
            if !block.owned {
                if alignment.get() > 0 {
                    // Determine padding required to align current block's index.
                    let alignment_padding =
                        (alignment.get() - (block.index % alignment.get())) % alignment.get();
                    // Properly aligned index of block.
                    let aligned_index = block.index + alignment_padding;
                    // Size of block, minus the bytes before the aligned index.
                    let aligned_size = block.size - alignment_padding;

                    // Ensure no overflow, in which case size is too small.
                    if aligned_size <= block.size {
                        if alignment_padding == 0 && block.size == size_in_bytes {
                            block.owned = true;
                        } else if aligned_size >= size_in_bytes {
                            if aligned_index > block.index {
                                // If our alignment forces us out-of-alignment with
                                // this block's  index, then allocate a block before to
                                // facilitate the unaligned size.
                                before_cursor
                                    .set(MemoryBlock {
                                        index: block.index,
                                        size: alignment_padding,
                                        owned: false,
                                    })
                                    .unwrap_or_else(|_| panic!(""));

                                block.size = block.size - alignment_padding;
                            }

                            after_cursor
                                .set(MemoryBlock {
                                    index: block.index + size_in_bytes,
                                    size: block.size - size_in_bytes,
                                    owned: false,
                                })
                                .unwrap_or_else(|_| panic!(""));

                            block.size = size_in_bytes;
                            block.owned = true;
                        }
                    }
                } else {
                    if block.size == size_in_bytes {
                        block.owned = true;
                    } else if block.size > size_in_bytes {
                        // Allocate new block after current with remaining length.
                        after_cursor
                            .set(MemoryBlock {
                                index: block.index + size_in_bytes,
                                size: block.size - size_in_bytes,
                                owned: false,
                            })
                            .unwrap_or_else(|_| panic!(""));

                        // Modify current block to reflect modified state.
                        block.size = size_in_bytes;
                        block.owned = true;
                    }
                }

                // If the block is now owned, we've successfully rented it out.
                if block.owned {
                    self.rented_bytes
                        .fetch_add(size_in_bytes, Ordering::AcqRel);
                    self.rented_blocks.fetch_add(1, Ordering::AcqRel);

                    unsafe {
                        if zero_memory {
                            std::ptr::write_bytes(self.head.add(block.index), 0, size_in_bytes);
                        }

                        memory_slice
                            .set(MemorySlice::<'a> {
                                pool: self,
                                index: block.index,
                                slice: &mut *std::slice::from_raw_parts_mut(
                                    self.head.add(block.index) as *mut _,
                                    size_in_bytes,
                                ),
                            })
                            .unwrap_or_else(|_| panic!(""));
                    }
                }
            }

            // Advance the cursor to the next item.
            map_cursor.move_next();
        }

        memory_slice.take().inspect(|_| {
            if let Some(before_block) = before_cursor.take() {
                map_cursor.insert_before(before_block);
            }

            if let Some(after_block) = after_cursor.take() {
                map_cursor.insert_after(after_block);
            }
        })
    }

    fn return_slice<'a, T>(&'a self, slice: &mut MemorySlice<'a, T>) {
        let mut map = self
            .map
            .lock()
            .expect("Memory pool map mutex has been poisoned!");

        let prev_unowned = OnceCell::new();
        let next_unowned = OnceCell::new();
        let mut slice_cursor = map.cursor_front_mut();
        while let Some(block) = slice_cursor.current() {
            if block.index == slice.index {
                self.rented_blocks.fetch_sub(1, Ordering::AcqRel);
                self.rented_bytes.fetch_sub(block.size, Ordering::AcqRel);

                if let Some(prev) = slice_cursor.peek_prev().filter(|a| !a.owned) {
                    prev_unowned.set((prev.index, prev.size)).unwrap();
                }

                if let Some(next) = slice_cursor.peek_next().filter(|a| !a.owned) {
                    next_unowned.set((next.index, next.size)).unwrap();
                }

                return;
            }

            slice_cursor.move_next();
        }

        if let Some((index, size)) = prev_unowned.get() {
            let mut block = slice_cursor.current().unwrap();
            block.index = *index;
            block.size += *size;

            slice_cursor.move_prev();
            slice_cursor.remove_current();
        }

        if let Some((_, size)) = next_unowned.get() {
            let mut block = slice_cursor.current().unwrap();
            block.size += *size;

            slice_cursor.move_next();
            slice_cursor.remove_current();
        }
    }
}
