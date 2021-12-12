mod buffer_allocator;

pub use buffer_allocator::*;

use super::OpenGLObject;
use std::mem::size_of;

bitflags::bitflags! {
    pub struct MapBufferAccessFlags : u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const INVALIDATE_RANGE = 1 << 2;
        const INVALIDATE_BUFFER = 1 << 3;
        const FLUSH_EXPLICIT = 1 << 4;
        const UNSYNCHRONIZED = 1 << 5;
        const PERSISTENT = 1 << 6;
        const COHERENT = 1 << 7;
    }
}

bitflags::bitflags! {
    pub struct BufferStorageFlags : u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const PERSISTENT = 1 << 6;
        const COHERENT = 1 << 7;
        const DYNAMIC = 1 << 8;
        const CLIENT = 1 << 9;
        const SPARSE = 1 << 10;
        const LGPU_SEPARATE_NVX = 1 << 11;
        const PER_GPU_NV = 1 << 11;
        const EXTERNAL_NVX = 1 << 13;

    }
}

#[repr(u32)]
pub enum BufferDraw {
    Stream = 35040,
    Static = 35044,
    Dynamic = 35048,
}

#[repr(u32)]
pub enum BufferTarget {
    Parameter = 33006,
    Array = 34962,
    Element = 34963,
    PixelPack = 35051,
    PixelUnpack = 35052,
    Uniform = 35345,
    Texture = 35882,
    TransformFeedback = 35982,
    CopyRead = 36662,
    CopyWrite = 36663,
    DrawIndirect = 36671,
    ShaderStorage = 37074,
    DispatchIndirect = 37102,
    Query = 37266,
    AtomicCounter = 37568,
}

pub struct Buffer<'s, T: Copy> {
    handle: u32,
    data_len: usize,
    data: Option<&'s mut [T]>,
}

impl<'s, T: Copy> Buffer<'s, T> {
    pub fn new() -> Self {
        let mut handle = 0;

        unsafe { gl::CreateBuffers(1, &raw mut handle) };

        Self {
            handle,
            data_len: 0,
            data: None,
        }
    }

    pub fn new_storage(data_len: usize, flags: BufferStorageFlags) -> Self {
        let mut buffer = Self::new();
        buffer.data_len = data_len;

        unsafe {
            gl::NamedBufferStorage(
                buffer.handle(),
                buffer.byte_len() as isize,
                std::ptr::null(),
                flags.bits(),
            )
        };

        buffer
    }

    pub fn new_data(data: &[T], draw: BufferDraw) -> Self {
        let mut buffer = Self::new();
        buffer.data_len = data.len();

        unsafe {
            gl::NamedBufferData(
                buffer.handle(),
                buffer.byte_len() as isize,
                data.as_ptr() as _,
                draw as u32,
            )
        };

        buffer
    }

    pub fn data_len(&self) -> usize {
        self.data_len
    }

    pub fn byte_len(&self) -> usize {
        self.data_len() * size_of::<T>()
    }

    fn data<'a>(&'a self) -> &'a [T] {
        self.data
            .as_ref()
            .expect("Cannot use buffer data when it isn't pinned.")
    }

    fn data_mut<'a>(&'a mut self) -> &'a mut [T] {
        self.data
            .as_mut()
            .expect("Cannot use buffer data when it isn't pinned.")
    }

    pub unsafe fn pin(&mut self, flags: MapBufferAccessFlags) {
        assert!(
            self.data_len() > 0,
            "Buffer length must be >0 to be pinned."
        );
        assert!(self.data.is_none(), "Buffer has already been pinned!");

        let ptr = gl::MapNamedBufferRange(self.handle(), 0, self.byte_len() as _, flags.bits());
        self.data = Some(std::slice::from_raw_parts_mut(
            ptr as *mut _,
            self.data_len(),
        ));
    }

    pub unsafe fn pin_range(
        &mut self,
        offset: isize,
        data_len: usize,
        flags: MapBufferAccessFlags,
    ) {
        assert!(
            data_len <= self.data_len(),
            "Range cannot exceed total buffer length."
        );

        let ptr = gl::MapNamedBufferRange(
            self.handle(),
            offset,
            (data_len * size_of::<T>()) as isize,
            flags.bits(),
        );
        self.data = Some(std::slice::from_raw_parts_mut(ptr as *mut _, data_len));
    }

    pub unsafe fn unpin(&mut self) {
        gl::UnmapNamedBuffer(self.handle());
        self.data = None;
    }

    pub fn resize_storage(&mut self, data_len: usize, flags: MapBufferAccessFlags) {
        assert!(
            self.data.is_none(),
            "Cannot resize buffer storage while buffer is pinned."
        );

        self.data_len = data_len;

        unsafe {
            gl::NamedBufferStorage(
                self.handle(),
                self.byte_len() as isize,
                std::ptr::null(),
                flags.bits(),
            )
        };
    }

    pub fn set_data(&mut self, data: &[T], draw: BufferDraw) {
        assert!(
            self.data.is_none(),
            "Cannot use `set_data` when buffer is pinned."
        );

        self.data_len = data.len();

        unsafe {
            gl::NamedBufferData(
                self.handle(),
                self.byte_len() as isize,
                data.as_ptr() as *const _,
                draw as u32,
            );

            self.pin(MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_BUFFER);
            self.data_mut().copy_from_slice(data);
            self.unpin();
        }
    }

    pub fn sub_data(&mut self, offset: isize, data: &[T]) {
        assert!(
            self.data.is_none(),
            "Cannot use `sub_data` when buffer is pinned."
        );

        unsafe {
            self.pin_range(
                offset,
                data.len(),
                MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_RANGE,
            );
            self.data_mut().copy_from_slice(data);
            self.unpin();
        }
    }

    pub fn bind(&self, target: BufferTarget) {
        unsafe { gl::BindBuffer(target as u32, self.handle()) };
    }
}

impl<T: Copy> crate::opengl::OpenGLObject for Buffer<'_, T> {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl<T: Copy> std::ops::Index<usize> for Buffer<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data()[index]
    }
}

impl<T: Copy> std::ops::IndexMut<usize> for Buffer<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data_mut()[index]
    }
}

impl<T: Copy> Drop for Buffer<'_, T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &raw const self.handle) };
    }
}
