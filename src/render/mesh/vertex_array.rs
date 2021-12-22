use crate::opengl::{
    buffer::{Buffer, RingBuffer},
    sync::RingFenceSync,
    VertexArrayObject,
};
use specs::{Component, HashMapStorage};

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct VertexArrayMesh {
    pub buffer: Buffer<f32>,
    pub vao: VertexArrayObject,
}

pub struct VertexArrayRenderSystem {
    draw_sync: RingFenceSync,
}

impl VertexArrayRenderSystem {
    pub fn new() -> Self {
        Self {
            draw_sync: RingFenceSync::new(2),
        }
    }
}

impl<'a> specs::System<'a> for VertexArrayRenderSystem {
    type SystemData = (
        specs::WriteExpect<'a, RingBuffer<crate::render::camera::CameraUniforms>>,
        specs::ReadExpect<'a, crate::AutomataWindow>,
        specs::ReadStorage<'a, crate::render::camera::Camera>,
        specs::ReadStorage<'a, crate::world::Transform>,
        specs::ReadStorage<'a, VertexArrayMesh>,
        specs::ReadStorage<'a, crate::render::Material>,
    );

    fn setup(&mut self, world: &mut specs::World) {
        use specs::{SystemData, WorldExt};

        Self::SystemData::setup(world);
        world.register::<VertexArrayMesh>();
    }

    fn run(
        &mut self,
        (mut view_uniforms, window, cameras, transforms, meshes, materials): Self::SystemData,
    ) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };

        use specs::Join;
        for camera in (&cameras).join() {
            if let Some(projector) = &camera.projector {
                view_uniforms.write(crate::render::camera::CameraUniforms::new(
                    window.viewport(),
                    projector.parameters(),
                    projector.matrix(),
                    camera.view,
                ));
                view_uniforms.bind(crate::opengl::buffer::BufferTarget::Uniform, 0);

                for (transform, mesh, maybe_material) in
                    (&transforms, &meshes, (&materials).maybe()).join()
                {
                    if let Some(material) = maybe_material {
                        material.pipeline.bind();
                        material
                            .pipeline
                            .vertex()
                            .set_uniform_mat4("model", transform.matrix)
                            .ok();
                    }
                    mesh.vao.bind();

                    self.draw_sync.wait_enter_next();
                    unsafe {
                        gl::DrawArrays(gl::TRIANGLES, 0, mesh.buffer.data_len() as i32);
                    }
                    self.draw_sync.fence_current();
                }

                view_uniforms.fence_current();
            }
        }
    }
}
