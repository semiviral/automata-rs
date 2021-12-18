pub mod buffer;
pub mod shader;
pub mod sync;
mod vertex_array_object;

use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
pub use vertex_array_object::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum GLError {
    NoError = 0,
    InvalidEnum = 1280,
    InvalidValue = 1281,
    InvalidOperation = 1282,
    StackOverflow = 1283,
    StackUnderflow = 1284,
    OutOfMemory = 1285,
    InvalidFramebufferOperation = 1286,
}

pub fn check_errors() {
    match GLError::try_from(unsafe { gl::GetError() }) {
        Ok(gl_error) if gl_error != GLError::NoError => {
            panic!("OpenGL error: {:?}", gl_error);
        }
        Err(err) => {
            panic!("Invalid OpenGL error code: {:?}", err)
        }
        _ => {}
    }
}

pub trait OpenGLObject {
    fn handle(&self) -> u32;
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DrawElementsType {
    u8 = 5121,
    u16 = 5123,
    u32 = 5125,
}
