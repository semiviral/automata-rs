use crate::opengl::OpenGLObject;
use std::collections::BTreeMap;

pub trait ShaderType {
    const SHADER_TYPE: u32;
}

pub enum Fragment {}
impl ShaderType for Fragment {
    const SHADER_TYPE: u32 = gl::FRAGMENT_SHADER;
}

pub enum Vertex {}
impl ShaderType for Vertex {
    const SHADER_TYPE: u32 = gl::VERTEX_SHADER;
}

pub enum Geometry {}
impl ShaderType for Geometry {
    const SHADER_TYPE: u32 = gl::GEOMETRY_SHADER;
}

pub enum TessellationEval {}
impl ShaderType for TessellationEval {
    const SHADER_TYPE: u32 = gl::TESS_EVALUATION_SHADER;
}

pub enum TessellationControl {}
impl ShaderType for TessellationControl {
    const SHADER_TYPE: u32 = gl::TESS_CONTROL_SHADER;
}

pub enum Compute {}
impl ShaderType for Compute {
    const SHADER_TYPE: u32 = gl::COMPUTE_SHADER;
}

pub struct ShaderProgram<T: ShaderType> {
    handle: u32,
    uniforms: BTreeMap<String, i32>,
    marker: std::marker::PhantomData<T>,
}

impl<T: ShaderType> ShaderProgram<T> {
    fn get_info_log(handle: u32) -> Option<String> {
        let mut log_len = 0;
        unsafe { gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &raw mut log_len) };

        if log_len > 0 {
            let mut log = vec![0; log_len as usize];
            unsafe {
                gl::GetProgramInfoLog(
                    handle,
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

    pub fn new(program_strings: &[&str]) -> Self {
        unsafe {
            let handle = gl::CreateShaderProgramv(
                T::SHADER_TYPE,
                program_strings.len() as i32,
                program_strings.as_ptr() as *const _,
            );

            crate::opengl::check_errors();
            if let Some(info_log) = Self::get_info_log(handle) {
                panic!("OpenGL failed to compile program object: {}", info_log);
            }

            let mut uniforms = BTreeMap::new();
            let mut uniform_count = -1;
            let mut max_uniform_len = -1;
            gl::GetProgramiv(handle, gl::ACTIVE_UNIFORMS, &raw mut uniform_count);
            gl::GetProgramiv(
                handle,
                gl::ACTIVE_UNIFORM_MAX_LENGTH,
                &raw mut max_uniform_len,
            );

            debug!("Identified {} uniforms for current shader.", uniform_count);
            let mut uniform_length = -1;
            for index in 0..(uniform_count as u32) {
                let mut name_buffer = Vec::<u8>::with_capacity(max_uniform_len as usize);
                let name_buffer_ptr = name_buffer.as_mut_ptr() as *mut gl::types::GLchar;

                gl::GetActiveUniformName(
                    handle,
                    index,
                    max_uniform_len,
                    &raw mut uniform_length,
                    name_buffer_ptr,
                );

                let name_len = uniform_length as usize;
                name_buffer.set_len(name_len);
                name_buffer.shrink_to_fit();
                uniforms.insert(
                    String::from_utf8(name_buffer)
                        .expect("Could not convert uniform name to string from buffer."),
                    gl::GetUniformLocation(handle, name_buffer_ptr),
                );
            }

            Self {
                handle,
                uniforms,
                marker: std::marker::PhantomData,
            }
        }
    }

    pub fn get_uniform(&self, name: &str, location: i32) -> Option<i32> {
        self.uniforms.get(&name.to_owned()).copied()
    }

    pub fn set_uniform_mat4(&self, name: &str, value: glam::Mat4) -> Result<(), ()> {
        match self.uniforms.get(name) {
            Some(location) => unsafe {
                gl::ProgramUniformMatrix4fv(
                    self.handle(),
                    1,
                    1,
                    false as u8,
                    &raw const value as *const _,
                );
                crate::opengl::check_errors();
                Ok(())
            },
            None => Err(()),
        }
    }

    pub fn get_uniforms(&self) -> std::collections::btree_map::Keys<String, i32> {
        self.uniforms.keys()
    }
}

impl<T: ShaderType> OpenGLObject for ShaderProgram<T> {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl<T: ShaderType> Drop for ShaderProgram<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.handle) };
    }
}
