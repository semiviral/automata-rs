use crate::opengl::{buffer::{RingBuffer, Buffer}, VertexArrayObject};
use specs::{Component, HashMapStorage, Join};

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct VertexArrayMesh {
    buffer: Buffer<f32>
    vao: VertexArrayObject,
}

pub struct VertexArrayRenderSystem;

impl<'a> specs::System<'a> for VertexArrayRenderSystem {
    type SystemData = (
        specs::WriteExpect<'a, RingBuffer<super::CameraUniforms>>,
        specs::ReadExpect<'a, crate::AutomataWindow>,
        specs::ReadStorage<'a, super::Camera>,
        specs::ReadStorage<'a, VertexArrayMesh>,
        specs::ReadStorage<'a, super::Material>,
    );

    fn run(&mut self, (mut view_uniforms, window, cameras, meshes, materials): Self::SystemData) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT) };

        use specs::Join;
        for camera in (&cameras).join() {
            if let Some(projector) = &camera.projector {
                view_uniforms.write(super::CameraUniforms {
                    viewport: window.viewport(),
                    parameters: projector.parameters(),
                    projection: projector.matrix(),
                    view: camera.view,
                });
                view_uniforms.bind(crate::opengl::buffer::BufferTarget::Uniform, 0);

                for (mesh, material) in (&meshes, (&materials).maybe()).join() {
                    material.program_pipeline.bind();
                    mesh.vao.bind();

                    unsafe {
                        gl::DrawArrays(gl::TRIANGLES, 0, mesh.)
                    }
                }

                view_uniforms.fence_current();
            }
        }
    }
}
