use crate::opengl::GLObject;
use std::collections::BTreeMap;

pub trait ShaderType {
    const GL_ENUM_VALUE: u32;
}

pub enum Fragment {}
impl ShaderType for Fragment {
    const GL_ENUM_VALUE: u32 = 35632;
}

pub enum Vertex {}
impl ShaderType for Vertex {
    const GL_ENUM_VALUE: u32 = 35633;
}

pub enum Geometry {}
impl ShaderType for Geometry {
    const GL_ENUM_VALUE: u32 = 36313;
}

pub enum TessellationEval {}
impl ShaderType for TessellationEval {
    const GL_ENUM_VALUE: u32 = 36487;
}

pub enum TessellationControl {}
impl ShaderType for TessellationControl {
    const GL_ENUM_VALUE: u32 = 36488;
}

pub enum Compute {}
impl ShaderType for Compute {
    const GL_ENUM_VALUE: u32 = 37305;
}

pub struct ShaderProgram<T: ShaderType> {
    handle: u32,
    uniforms: BTreeMap<String, i32>,
    marker: std::marker::PhantomData<T>,
}

impl<T: ShaderType> ShaderProgram<T> {
    pub fn new(program_strings: &[&str]) -> Self {
        unsafe {
            let handle = gl::CreateShaderProgramv(
                T::GL_ENUM_VALUE,
                program_strings.len() as i32,
                program_strings as *const _ as *const _,
            );

            let mut uniforms = BTreeMap::new();
            let mut uniform_count = -1;
            let mut max_uniform_len = -1;
            gl::GetProgramiv(handle, gl::ACTIVE_UNIFORMS, &raw mut uniform_count);
            gl::GetProgramiv(
                handle,
                gl::ACTIVE_UNIFORM_MAX_LENGTH,
                &raw mut max_uniform_len,
            );

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

                let name_len = (uniform_length + 1) as usize;
                name_buffer.set_len(name_len);
                name_buffer.shrink_to(name_len);
                uniforms.insert(
                    String::from_utf8(name_buffer)
                        .expect("Could not convert uniform name to string from buffer."),
                    gl::GetUniformLocation(handle, name_buffer_ptr),
                );
            }

            let _self = Self {
                handle,
                uniforms,
                marker: std::marker::PhantomData,
            };

            if let Some(log) = _self.get_info_log() {
                panic!("OpenGL object info log: {}", log);
            }

            _self
        }
    }
}

impl<T: ShaderType> GLObject for ShaderProgram<T> {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl<T: ShaderType> Drop for ShaderProgram<T> {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.handle) };
    }
}
