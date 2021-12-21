mod camera;
mod material;
pub mod mesh;

pub use camera::*;
pub use material::*;

use glam::{Mat4, Vec4};

#[repr(C)]
pub struct CameraUniforms {
    viewport: Vec4,
    parameters: Vec4,
    projection: Mat4,
    view: Mat4,
}

pub struct OpenGLMaintenanceSystem;

impl<'a> specs::System<'a> for OpenGLMaintenanceSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}