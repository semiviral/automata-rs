mod material;

pub mod camera;
pub mod mesh;

pub use material::*;

use glam::{Mat4, Vec4};

pub struct OpenGLMaintenanceSystem;

impl<'a> specs::System<'a> for OpenGLMaintenanceSystem {
    type SystemData = ();

    fn run(&mut self, data: Self::SystemData) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }
}
