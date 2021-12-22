use glam::Vec2;
use specs::{Component, HashMapStorage};

#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct InputVector {
    pub keyb: Vec2,
    pub sensitivity: f32,
}

pub struct InputVectorTranslationSystem;
impl<'a> specs::System<'a> for InputVectorTranslationSystem {
    type SystemData = (
        specs::ReadExpect<'a, crate::time::DeltaTime>,
        specs::ReadStorage<'a, InputVector>,
        specs::WriteStorage<'a, crate::world::Transform>,
    );

    fn run(&mut self, (delta, input, mut transform): Self::SystemData) {
        use specs::Join;

        for (input, transform) in (&input, &mut transform).join() {
            if input.keyb == Vec2::ZERO {
                continue;
            } else {
                let delta_sensitivity = delta.0.as_secs_f32() * input.sensitivity;

                transform.pos += transform.rot.mul_vec3(-glam::Vec3::new(
                    delta_sensitivity * input.keyb.x,
                    0.0,
                    delta_sensitivity * input.keyb.y,
                ));
            }
        }
    }
}
