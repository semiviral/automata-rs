use glam::Mat4;
use specs::{Component, DenseVecStorage};

pub mod block;
pub mod chunk;

#[derive(Debug, Component)]
#[storage(DenseVecStorage)]
pub struct Transform {
    pub pos: glam::Vec3,
    pub rot: glam::Quat,
    pub scale: glam::Vec3,
    pub matrix: Mat4,
}

impl Transform {
    pub fn matrix_srt(&self) -> Mat4 {
        Mat4::from_scale(self.scale) * Mat4::from_quat(self.rot) * Mat4::from_translation(self.pos)
    }

    pub fn matrix_trs(&self) -> Mat4 {
        Mat4::from_translation(self.pos) * Mat4::from_quat(self.rot) * Mat4::from_scale(self.scale)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            pos: glam::Vec3::new(0.0, 0.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::new(1.0, 1.0, 1.0),
            matrix: glam::Mat4::IDENTITY,
        }
    }
}

pub struct TransformMatrixSystem;

impl<'a> specs::System<'a> for TransformMatrixSystem {
    type SystemData = (
        specs::WriteStorage<'a, crate::world::Transform>,
        specs::WriteStorage<'a, crate::render::camera::Camera>,
    );

    fn run(&mut self, (mut transforms, mut cameras): Self::SystemData) {
        use specs::Join;
        for (transform, maybe_camera) in (&mut transforms, (&mut cameras).maybe()).join() {
            if let Some(camera) = maybe_camera {
                transform.matrix = transform.matrix_trs();
                camera.view = transform.matrix.inverse();
            } else {
                transform.matrix = transform.matrix_srt();
            }
        }
    }
}
