use super::ShaderProgram;
use crate::opengl::OpenGLObject;

pub struct ProgramPipeline {
    handle: u32,
    vertex: ShaderProgram<super::Vertex>,
    tess_ctrl: Option<ShaderProgram<super::TessellationControl>>,
    tess_eval: Option<ShaderProgram<super::TessellationEval>>,
    geometry: Option<ShaderProgram<super::Geometry>>,
    fragment: Option<ShaderProgram<super::Fragment>>,
}

impl ProgramPipeline {
    pub fn new(
        vertex_shader: ShaderProgram<super::Vertex>,
        tess_ctrl_shader: Option<ShaderProgram<super::TessellationControl>>,
        tess_eval_shader: Option<ShaderProgram<super::TessellationEval>>,
        geometry_shader: Option<ShaderProgram<super::Geometry>>,
        fragment_shader: Option<ShaderProgram<super::Fragment>>,
    ) -> Self {
        let mut handle = 0;
        unsafe {
            gl::CreateProgramPipelines(1, &raw mut handle);

            gl::UseProgramStages(handle, gl::VERTEX_SHADER_BIT, vertex_shader.handle());

            if let Some(tess_ctrl) = &tess_ctrl_shader {
                gl::UseProgramStages(handle, gl::TESS_CONTROL_SHADER_BIT, tess_ctrl.handle());
            }

            if let Some(tess_eval) = &tess_eval_shader {
                gl::UseProgramStages(handle, gl::TESS_EVALUATION_SHADER_BIT, tess_eval.handle());
            }

            if let Some(geometry) = &geometry_shader {
                gl::UseProgramStages(handle, gl::GEOMETRY_SHADER_BIT, geometry.handle());
            }

            if let Some(fragment) = &fragment_shader {
                gl::UseProgramStages(handle, gl::FRAGMENT_SHADER_BIT, fragment.handle());
            }

            crate::opengl::check_errors();
        }

        Self {
            handle,
            vertex: vertex_shader,
            tess_ctrl: tess_ctrl_shader,
            tess_eval: tess_eval_shader,
            geometry: geometry_shader,
            fragment: fragment_shader,
        }
    }

    pub fn bind(&self) {
        unsafe { gl::BindProgramPipeline(self.handle()) };
    }
}

impl crate::opengl::OpenGLObject for ProgramPipeline {
    fn handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for ProgramPipeline {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgramPipelines(1, [self.handle].as_ptr()) };
    }
}
