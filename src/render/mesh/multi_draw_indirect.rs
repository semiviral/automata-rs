use gl::DrawArraysIndirect;
use mint::RowMatrix4;

use crate::opengl::{
    buffer::{Buffer, BufferAllocator, MapBufferAccessFlags},
    sync::FenceSync,
    VertexArrayObject,
};

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

pub struct MultiDrawIndirectMesh<'a> {
    commands: Buffer<'a, DrawElementsIndirectCommand>,
    next_command_index: u32,
    buffer_allocator: BufferAllocator,
    vertex_array_obj: VertexArrayObject,
    models: Buffer<'a, RowMatrix4<f32>>,
    next_model_index: u32,
    draw_type: crate::opengl::DrawElementsType,
    draw_sync: FenceSync,
}

impl MultiDrawIndirectMesh<'_> {
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
            use crate::opengl::buffer::MapBufferAccessFlags;

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

    pub fn push_model(&mut self, model: RowMatrix4<f32>) {
        self.models[self.next_model_index as usize] = model;
        self.next_model_index += 1;
    }

    pub fn clear_vertex_attribs(&mut self) {
        self.vertex_array_obj.clear_vertex_attributes();
    }

    pub fn push_vertex_attrib(
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

impl super::Mesh for MultiDrawIndirectMesh<'_> {
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
