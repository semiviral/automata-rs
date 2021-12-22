use crate::opengl::{
    buffer::{Buffer, BufferAllocator, MapBufferAccessFlags, RingBuffer},
    sync::FenceSync,
    VertexArrayObject,
};
use glam::Mat4;
use specs::{Component, HashMapStorage};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DrawElementsIndirectCommand {
    index_count: u32,
    instance_count: u32,
    first_index_offset: u32,
    first_vertex_offset: u32,
    base_instance: u32,
}

impl DrawElementsIndirectCommand {
    pub const fn new(
        index_count: u32,
        instance_count: u32,
        first_index_offset: u32,
        first_vertex_offset: u32,
        base_instance: u32,
    ) -> Self {
        Self {
            index_count,
            instance_count,
            first_index_offset,
            first_vertex_offset,
            base_instance,
        }
    }
}

#[derive(Component)]
#[storage(HashMapStorage)]
pub struct MultiDrawIndirectMesh {
    commands: Buffer<DrawElementsIndirectCommand>,
    next_command_index: u32,
    buffer_allocator: BufferAllocator,
    vertex_array_obj: VertexArrayObject,
    models: Buffer<glam::Mat4>,
    next_model_index: u32,
    draw_type: crate::opengl::DrawElementsType,
    draw_sync: FenceSync,
}

impl MultiDrawIndirectMesh {
    pub fn new(allocator_size: usize, draw_type: crate::opengl::DrawElementsType) -> Self {
        const MAX_MODEL_COUNT: usize = 21000;

        let commands = Buffer::new();
        let buffer_allocator = BufferAllocator::new(allocator_size);
        let mut vertex_array_obj = VertexArrayObject::new();
        let models = Buffer::new_storage(
            MAX_MODEL_COUNT,
            crate::opengl::buffer::BufferStorageFlags::WRITE,
        );

        vertex_array_obj.allocate_vertex_buffer_binding(0, &buffer_allocator, 0, 0);
        vertex_array_obj.allocate_vertex_buffer_binding(1, &models, 0, 1);

        Self {
            commands,
            next_command_index: 0,
            buffer_allocator,
            vertex_array_obj,
            models,
            next_model_index: 0,
            draw_type,
            draw_sync: FenceSync::new(0),
        }
    }

    pub fn prepare_draw(&mut self, command_count: u32) {
        if (command_count as usize) > self.commands.data_len() {
            self.commands.resize_storage(
                command_count as usize,
                MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_BUFFER,
            );
        }

        self.next_command_index = 0;
        self.next_model_index = 0;

        unsafe {
            self.commands.pin_range(
                0,
                command_count as usize,
                MapBufferAccessFlags::WRITE | MapBufferAccessFlags::INVALIDATE_RANGE,
            )
        };
    }

    pub fn push_draw_command(&mut self, command: DrawElementsIndirectCommand) {
        assert!(
            self.next_command_index < u32::MAX,
            "MultiDrawIndirectMesh has reached maximum possible count of draw commands."
        );

        self.commands[self.next_command_index as usize] = command;
        self.next_command_index += 1;
    }

    pub fn push_model(&mut self, model: glam::Mat4) {
        self.models[self.next_model_index as usize] = model;
        self.next_model_index += 1;
    }

    pub fn clear_vertex_attribs(&mut self) {
        self.vertex_array_obj.clear_vertex_attributes();
    }

    pub fn push_vertex_attrib(
        &mut self,
        index: u32,
        dimensions: u32,
        offset: u32,
        binding_index: u32,
        format: crate::opengl::VertexFormat,
    ) {
        self.vertex_array_obj.allocate_vertex_attribute(
            index,
            dimensions,
            offset,
            binding_index,
            format,
        );
    }

    pub fn commit_vao(&mut self) {
        self.vertex_array_obj.commit(Some(&self.buffer_allocator));
    }

    pub fn rent_element_memory<'a, T>(
        &'a mut self,
        len: std::num::NonZeroUsize,
        alignment: std::num::NonZeroUsize,
        zero_memory: bool,
    ) -> Option<crate::memory::MemorySlice<'a, T>> {
        self.buffer_allocator
            .rent_slice(len, alignment, zero_memory)
    }
    fn visible(&self) -> bool {
        true
    }

    fn draw(&mut self) {
        self.draw_sync.busy_wait_cpu();

        self.vertex_array_obj.bind();
        self.commands
            .bind(crate::opengl::buffer::BufferTarget::DrawIndirect);

        unsafe {
            gl::MultiDrawElementsIndirect(
                gl::TRIANGLES,
                self.draw_type as _,
                std::ptr::null(),
                self.commands.data_len() as i32,
                0,
            )
        };

        self.draw_sync.regenerate(0);
    }
}

#[repr(C)]
struct ModelUniforms {
    mvp: Mat4,
    obj: Mat4,
    world: Mat4,
}

pub struct MultiDrawIndirectRenderSystem {
    model_uniform: RingBuffer<ModelUniforms>,
}

impl MultiDrawIndirectRenderSystem {
    pub fn new() -> Self {
        unsafe {
            gl::FrontFace(gl::CCW);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::CULL_FACE);

            let mut alignment = 0;
            gl::GetIntegerv(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT, &raw mut alignment);

            Self {
                model_uniform: RingBuffer::new(8, alignment as usize),
            }
        }
    }
}

impl<'a> specs::System<'a> for MultiDrawIndirectRenderSystem {
    type SystemData = (
        specs::WriteExpect<'a, RingBuffer<crate::render::camera::CameraUniforms>>,
        specs::ReadExpect<'a, crate::AutomataWindow>,
        specs::ReadStorage<'a, crate::render::camera::Camera>,
    );

    fn setup(&mut self, world: &mut specs::World) {
        use specs::{SystemData, WorldExt};

        Self::SystemData::setup(world);
        world.register::<MultiDrawIndirectMesh>();
    }

    fn run(&mut self, (mut view_uniforms, window, cameras): Self::SystemData) {
        // TODO clip frustum

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
                // draw_models
                view_uniforms.fence_current();
            }
        }
    }
}
