use specs::{Component, HashMapStorage};

use crate::opengl::shader::ProgramPipeline;

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct Material {
    pub pipeline: ProgramPipeline,
    // textures TODO
}
