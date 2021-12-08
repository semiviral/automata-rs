pub mod shader;

pub trait GLObject {
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
