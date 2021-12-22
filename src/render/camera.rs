use glam::{Mat4, Vec4};
use specs::{Component, HashMapStorage};
use std::f32::consts::PI;

pub enum ProjectorMode {
    Prespective,
    Orthographic,
}

pub struct Projector {
    matrix: Mat4,
    parameters: Vec4,
}

impl Projector {
    pub fn new_perspective(fov: f32, aspect_ratio: f32, near_clip: f32, far_clip: f32) -> Self {
        assert!(
            fov > 0.0 && fov < 360.0,
            "Field of view must be a valid 360 degree value."
        );
        assert!(near_clip > 0.0, "Near clip must be a positive distance.");
        assert!(far_clip > 0.0, "Far clip must be a positive distance.");
        assert!(
            near_clip < far_clip,
            "Near clip must be less than far clip."
        );

        let fov_radians = fov * (PI / 180.0);
        let y_scale = 1.0 / f32::tan(fov_radians * 0.5);
        let x_scale = y_scale / aspect_ratio;
        let neg_far_range = if f32::is_infinite(far_clip) {
            -1.0
        } else {
            far_clip / (near_clip - far_clip)
        };

        let mut result = Mat4::ZERO;
        *result.col_mut(0) = Vec4::new(x_scale, 0.0, 0.0, 0.0);
        *result.col_mut(1) = Vec4::new(0.0, y_scale, 0.0, 0.0);
        *result.col_mut(2) = Vec4::new(0.0, 0.0, neg_far_range, -1.0);
        *result.col_mut(3) = Vec4::new(0.0, 0.0, near_clip * neg_far_range, 1.0);

        Self {
            matrix: result,
            parameters: Vec4::new(fov, aspect_ratio, near_clip, far_clip),
        }
    }

    pub fn new_orthrographic(width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        let mut result = Mat4::IDENTITY;
        let z_difference = z_near - z_far;

        result.col_mut(0)[0] = 2.0 / width;
        result.col_mut(1)[1] = 2.0 / height;
        result.col_mut(2)[2] = 1.0 / z_difference;
        result.col_mut(3)[2] = z_near / z_difference;

        Self {
            matrix: result,
            parameters: Vec4::new(width, height, z_near, z_far),
        }
    }

    pub fn parameters(&self) -> Vec4 {
        self.parameters
    }

    pub fn matrix(&self) -> Mat4 {
        self.matrix
    }
}

#[repr(C)]
pub struct CameraUniforms(Vec4, Vec4, Mat4, Mat4);

impl CameraUniforms {
    pub const fn new(viewport: Vec4, parameters: Vec4, projection: Mat4, view: Mat4) -> Self {
        Self(viewport, parameters, projection, view)
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Camera {
    pub view: Mat4,
    pub projector_mode: ProjectorMode,
    pub projector: Option<Projector>,
}
