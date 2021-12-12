use mint::RowMatrix4;

use crate::opengl::{
    buffer::{Buffer, BufferAllocator, OpenGLBuffer},
    VertexArrayObject,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DrawElementsIndirectCommand {
    index_count: u32,
    instance_count: u32,
    first_index_offset: u32,
    first_vertex_offset: u32,
    base_instance: u32,
}

pub struct MultidrawIndirectMesh {
    commands: Buffer<DrawElementsIndirectCommand>,
    buffer_allocator: BufferAllocator,
    vertex_array_obj: VertexArrayObject,
    models: Buffer<RowMatrix4<f32>>,
    draw_type: crate::opengl::DrawElementsType,
    draw_sync: crate::opengl::sync::FenceSync
}

impl MultidrawIndirectMesh {
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
            buffer_allocator,
            vertex_array_obj,
            models,
            draw_type,
            draw_sync: crate::opengl::sync::FenceSync::new(0)
        }
    }

    pub fn clear_vertex_attribs(&mut self) {
        self.vertex_array_obj.clear_vertex_attributes();
    }

    pub fn allocate_vertex_attrib(
        &mut self,
        vertex_attribs: Vec<Box<dyn crate::opengl::VertexAttribute>>,
    ) {
        for vertex_attrib in vertex_attribs {
            self.vertex_array_obj
                .allocate_vertex_attribute(vertex_attrib);
        }
    }

    pub fn commit_vao(&mut self) {
        self.vertex_array_obj.commit(Some(&self.buffer_allocator));
    }

    pub fn commit_draw_commands(&mut self, commands: &[DrawElementsIndirectCommand]) {
        self.commands
            .set_data(commands, crate::opengl::buffer::BufferDraw::Static);
    }

    pub fn commit_models(&mut self, models: &[RowMatrix4<f32>]) {
        self.models.sub_data(0, models);
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
}

impl super::Mesh for MultidrawIndirectMesh {
    fn visible(&self) -> bool {
        true
    }

    fn draw(&self) {
        self.vertex_array_obj.bind();
        self.commands.bind(crate::opengl::buffer::BufferTarget::DrawIndirect);

        unsafe {
            gl::MultiDraw
        }
    }
}