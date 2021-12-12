use crate::{memory::MemoryPool, opengl::OpenGLObject};

pub struct BufferAllocator {
    handle: u32,
    pool: MemoryPool,
}

impl BufferAllocator {
    pub fn new(size: usize) -> Self {
        unsafe {
            let mut handle = 0;
            gl::CreateBuffers(1, &raw mut handle);
            gl::NamedBufferStorage(
                handle,
                size as isize,
                std::ptr::null(),
                (gl::MAP_WRITE_BIT | gl::MAP_PERSISTENT_BIT | gl::MAP_COHERENT_BIT) as u32,
            );

            Self {
                handle,
                pool: MemoryPool::new(gl::MapNamedBuffer(handle, gl::WRITE_ONLY) as *mut _, size),
            }
        }
    }

    pub fn rent_slice<'a, T>(
        &'a self,
        size: std::num::NonZeroUsize,
        alignment: std::num::NonZeroUsize,
        zero_memory: bool,
    ) -> Option<crate::memory::MemorySlice<'a, T>> {
        self.pool.rent_slice(size, alignment, zero_memory)
    }
}

impl OpenGLObject for BufferAllocator {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for BufferAllocator {
    fn drop(&mut self) {
        unsafe {
            gl::UnmapNamedBuffer(self.handle());
            gl::DeleteBuffers(1, &raw const self.handle);
        }
    }
}
