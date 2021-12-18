use std::mem::size_of;

use crate::opengl::sync::RingFenceSync;

pub struct RingBuffer<T> {
    handle: u32,
    ring_sync: RingFenceSync,
    aligned_size: usize,
    ptr: *mut std::ffi::c_void,
    marker: std::marker::PhantomData<T>,
}

impl<T> RingBuffer<T> {
    pub fn new(buffer_count: usize, alignment: usize) -> Self {
        let aligned_size = size_of::<T>() + (size_of::<T>() % alignment);

        unsafe {
            let mut handle = 0;
            gl::CreateBuffers(1, &raw mut handle);

            use super::BufferStorageFlags;
            let total_size = aligned_size * buffer_count;
            gl::NamedBufferStorage(
                handle,
                total_size as isize,
                std::ptr::null(),
                (BufferStorageFlags::WRITE
                    | BufferStorageFlags::PERSISTENT
                    | BufferStorageFlags::COHERENT)
                    .bits(),
            );

            Self {
                handle,
                ring_sync: RingFenceSync::new(buffer_count),
                aligned_size,
                ptr: gl::MapNamedBuffer(handle, gl::WRITE_ONLY),
                marker: std::marker::PhantomData,
            }
        }
    }

    pub fn write(&mut self, val: T) {
        self.ring_sync.wait_enter_next();
        unsafe {
            (self.ptr.add(self.ring_sync.index() * self.aligned_size) as *mut T).write(val);
        }
    }

    pub fn fence_current(&mut self) {
        self.ring_sync.fence_current()
    }

    pub fn bind(&self, target: super::BufferTarget, binding_index: u32) {
        unsafe {
            use crate::opengl::OpenGLObject;
            gl::BindBufferRange(
                target as u32,
                binding_index,
                self.handle(),
                (self.ring_sync.index() * size_of::<T>()) as isize,
                size_of::<T>() as isize,
            );
        }
    }
}

impl<T> crate::opengl::OpenGLObject for RingBuffer<T> {
    fn handle(&self) -> u32 {
        self.handle
    }
}
