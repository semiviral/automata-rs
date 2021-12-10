use std::{
    collections::{linked_list::Cursor, LinkedList},
    lazy::OnceCell,
    num::NonZeroUsize,
    sync::Mutex,
};

pub struct MemorySlice<'a, T> {
    index: usize,
    slice: &'a mut [T],
}

struct MemoryBlock {
    index: usize,
    size: usize,
    owned: bool,
}

pub struct MemoryPool {
    head: *mut u8,
    total_bytes: usize,
    map: Mutex<LinkedList<MemoryBlock>>,
    rented_bytes: usize,
    rented_blocks: usize,
}

impl MemoryPool {
    pub fn new(head: *mut u8, byte_len: usize) -> Self {
        Self {
            head,
            total_bytes: byte_len,
            map: Mutex::new(LinkedList::new()),
            rented_bytes: 0,
            rented_blocks: 0,
        }
    }

    pub fn remaining_bytes(&self) -> usize {
        self.total_bytes - self.rented_bytes
    }

    pub fn rent<'a, T>(
        &'a mut self,
        size: NonZeroUsize,
        alignment: NonZeroUsize,
        zero_memory: bool,
    ) -> Option<MemorySlice<'a, T>> {
        let mut memory_slice = OnceCell::new();
        let mut map = self
            .map
            .lock()
            .expect("Memory pool map mutex has been poisoned!");
        let size_in_bytes = size.get() * std::mem::size_of::<T>();

        if size_in_bytes <= self.remaining_bytes() {
            let mut map_cursor = map.cursor_front_mut();

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
                                    map_cursor.insert_before(MemoryBlock {
                                        index: block.index,
                                        size: alignment_padding,
                                        owned: false,
                                    });

                                    block.size = block.size - alignment_padding;
                                }

                                map_cursor.insert_after(MemoryBlock {
                                    index: block.index + size_in_bytes,
                                    size: block.size - size_in_bytes,
                                    owned: false,
                                });

                                block.size = size_in_bytes;
                                block.owned = true;
                            }
                        }
                    } else {
                        if block.size == size_in_bytes {
                            block.owned = true;
                        } else if block.size > size_in_bytes {
                            // Allocate new block after current with remaining length.
                            map_cursor.insert_after(MemoryBlock {
                                index: block.index + size_in_bytes,
                                size: block.size - size_in_bytes,
                                owned: false,
                            });

                            // Modify current block to reflect modified state.
                            block.size = size_in_bytes;
                            block.owned = true;
                        }
                    }

                    // If the block is now owned, we've successfully rented it out.
                    if block.owned {
                        unsafe {
                            let block_start_ptr = self.head.add(block.index);

                            if zero_memory {
                                std::ptr::write_bytes(block_start_ptr, 0, size_in_bytes);
                            }

                            memory_slice
                                .set(MemorySlice::<'a> {
                                    index: block.index,
                                    slice: &mut *std::slice::from_raw_parts_mut(
                                        self.head.add(block.index) as *mut _,
                                        size_in_bytes,
                                    ),
                                })
                                .ok();
                        }

                        self.rented_bytes += size_in_bytes;
                        self.rented_blocks += 1;
                        break;
                    }
                }

                map_cursor.move_next();
            }
        }

        memory_slice.take()
    }
}
