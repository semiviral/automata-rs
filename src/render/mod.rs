mod camera;
mod material;
mod multi_draw_indirect;
mod vertex_array;

pub use camera::*;
pub use material::*;
pub use multi_draw_indirect::*;
pub use vertex_array::*;

use crate::opengl::buffer::RingBuffer;
use glam::{Mat4, Vec4};

#[repr(C)]
pub struct CameraUniforms {
    viewport: Vec4,
    parameters: Vec4,
    projection: Mat4,
    view: Mat4,
}
