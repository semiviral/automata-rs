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

pub trait OpenGLBuffer<T: Copy>: OpenGLObject {
    fn len(&self) -> usize;
    fn len_mut(&mut self) -> &mut usize;

    unsafe fn pin(&self, flags: MapBufferAccessFlags) -> &mut [T] {
        assert!(self.len() > 0, "Buffer length must be >0 to be pinned.");
        assert!(
            (self.len() % size_of::<T>()) == 0,
            "Buffer storage range must be aligned to `size_of::<T>()`"
        );

        let ptr = gl::MapNamedBufferRange(self.handle(), 0, self.len() as _, flags.bits());
        std::slice::from_raw_parts_mut(ptr as *mut _, self.len() / size_of::<T>())
    }

    unsafe fn pin_range(
        &self,
        offset: isize,
        length: usize,
        flags: MapBufferAccessFlags,
    ) -> &mut [T] {
        assert!(
            length <= self.len(),
            "Range cannot exceed total buffer length."
        );

        let ptr = gl::MapNamedBufferRange(
            self.handle(),
            offset,
            (length * size_of::<T>()) as isize,
            flags.bits(),
        );

        std::slice::from_raw_parts_mut(ptr as *mut _, length)
    }

    unsafe fn unpin(&self) {
        gl::UnmapNamedBuffer(self.handle());
    }

    fn set_data(&mut self, data: &[T], draw: BufferDraw) {
        *self.len_mut() = data.len() * size_of::<T>();

        unsafe {
            gl::NamedBufferData(
                self.handle(),
                self.len() as isize,
                data.as_ptr() as *const _,
                draw as u32,
            );

            let slice =
                self.pin(MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_BUFFER);

            slice.copy_from_slice(data);

            self.unpin();
        }
    }

    fn sub_data(&mut self, offset: isize, data: &[T]) {
        unsafe {
            let slice = self.pin_range(
                offset,
                data.len(),
                MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_RANGE,
            );

            slice.copy_from_slice(data);

            self.unpin();
        }
    }

    fn bind(&self, target: BufferTarget) {
        unsafe { gl::BindBuffer(target as u32, self.handle()) };
    }
}

pub struct Buffer<T: Copy> {
    handle: u32,
    len: usize,
    marker: std::marker::PhantomData<T>,
}

impl<T: Copy> Buffer<T> {
    pub fn new() -> Self {
        let mut handle = 0;

        unsafe { gl::CreateBuffers(1, &raw mut handle) };

        Self {
            handle,
            len: 0,
            marker: std::marker::PhantomData,
        }
    }

    pub fn new_storage(len: usize, flags: BufferStorageFlags) -> Self {
        let byte_length = len * size_of::<T>();
        let mut buffer = Self::new();
        buffer.len = byte_length;

        unsafe {
            gl::NamedBufferStorage(
                buffer.handle(),
                byte_length as isize,
                std::ptr::null(),
                flags.bits(),
            )
        };

        buffer
    }

    pub fn new_data(data: &[T], draw: BufferDraw) -> Self {
        let byte_length = data.len() * size_of::<T>();
        let mut buffer = Self::new();
        buffer.len = byte_length;

        unsafe {
            gl::NamedBufferData(
                buffer.handle(),
                byte_length as isize,
                data.as_ptr() as _,
                draw as u32,
            )
        };

        buffer
    }
}

impl<T: Copy> crate::opengl::OpenGLObject for Buffer<T> {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl<T: Copy> OpenGLBuffer<T> for Buffer<T> {
    fn len(&self) -> usize {
        self.len
    }

    fn len_mut(&mut self) -> &mut usize {
        &mut self.len
    }
}

impl<T: Copy> Drop for Buffer<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteBuffers(1, &raw const self.handle) };
    }
}
