use crate::opengl::GLObject;

use super::{Fragment, ShaderProgram, Vertex};

pub struct ProgramPipeline {
    handle: u32,
    vertex: ShaderProgram<Vertex>,
    fragment: ShaderProgram<Fragment>,
}

impl ProgramPipeline {
    pub fn new(
        vertex_shader: ShaderProgram<Vertex>,
        fragment_shader: ShaderProgram<Fragment>,
    ) -> Self {
        let mut handle = 0;
        unsafe {
            gl::CreateProgramPipelines(1, &raw mut handle);
            gl::UseProgramStages(handle, gl::VERTEX_SHADER_BIT, vertex_shader.handle());
            gl::UseProgramStages(handle, gl::FRAGMENT_SHADER_BIT, fragment_shader.handle());
        }

        let _self = Self {
            handle,
            vertex: vertex_shader,
            fragment: fragment_shader,
        };

        if let Some(log) = _self.get_info_log() {
            panic!("OpenGL object info log: {}", log);
        }

        _self
    }
}

impl crate::opengl::GLObject for ProgramPipeline {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for ProgramPipeline {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgramPipelines(1, [self.handle].as_ptr()) };
    }
}