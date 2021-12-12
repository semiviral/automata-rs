pub mod buffer;
pub mod shader;
pub mod sync;
mod vertex_array_object;

pub use vertex_array_object::*;

pub trait OpenGLObject {
    fn handle(&self) -> u32;

    fn get_info_log(&self) -> Option<String> {
        let mut log_len = 0;
        unsafe { gl::GetProgramiv(self.handle(), gl::INFO_LOG_LENGTH, &raw mut log_len) };

        if log_len > 0 {
            let mut log = vec![0; log_len as usize];
            unsafe {
                gl::GetProgramInfoLog(
                    self.handle(),
                    log.len() as i32,
                    &raw mut log_len,
                    log.as_mut_ptr() as *mut _,
                )
            };

            Some(
                String::from_utf8(log)
                    .expect("Failed to convert info log bytes into a valid UTF-8 string."),
            )
        } else {
            None
        }
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum DrawElementsType {
    u8 = 5121,
    u16 = 5123,
    u32 = 5125,
}
