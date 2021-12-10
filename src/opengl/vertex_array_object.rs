use std::collections::BTreeMap;

use super::OpenGLObject;

pub trait VertexAttribute {
    fn index(&self) -> u32;
    fn dimensions(&self) -> i32;
    fn offset(&self) -> u32;
    fn binding_index(&self) -> u32;
    fn normalized(&self) -> bool;
    fn stride(&self) -> u32;

    fn commit_vao_format(&self, vao_handle: u32);
}

pub struct PackedVertexAttribute {
    index: u32,
    dimensions: i32,
    offset: u32,
    binding_index: u32,
    normalized: bool,
    stride: u32,
}

impl VertexAttribute for PackedVertexAttribute {
    fn index(&self) -> u32 {
        self.index
    }

    fn dimensions(&self) -> i32 {
        self.dimensions
    }

    fn offset(&self) -> u32 {
        self.offset
    }

    fn binding_index(&self) -> u32 {
        self.binding_index
    }

    fn normalized(&self) -> bool {
        self.normalized
    }

    fn stride(&self) -> u32 {
        self.stride
    }

    fn commit_vao_format(&self, vao_handle: u32) {
        unsafe {
            gl::VertexArrayAttribIFormat(
                vao_handle,
                self.index(),
                self.dimensions(),
                gl::UNSIGNED_INT,
                self.offset(),
            )
        };
    }
}

struct VertexBufferObjectBinding {
    handle: u32,
    vertex_offset: isize,
    divisor: u32,
}

pub struct VertexArrayObject {
    handle: u32,
    vertex_attribs: Vec<Box<dyn VertexAttribute>>,
    vertex_buffer_bindings: BTreeMap<u32, VertexBufferObjectBinding>,
}

impl VertexArrayObject {
    pub fn clear_vertex_attributes(&mut self) {
        self.vertex_attribs.clear();
    }

    pub fn allocate_vertex_attribute(&mut self, vertex_attrib: Box<dyn VertexAttribute>) {
        self.vertex_attribs.push(vertex_attrib);
    }

    pub fn allocate_vertex_buffer_binding(
        &mut self,
        binding_index: u32,
        buffer: Box<dyn OpenGLObject>,
        vertex_offset: isize,
        divisor: u32,
    ) {
        let vertex_buffer_binding = VertexBufferObjectBinding {
            handle: buffer.handle(),
            vertex_offset,
            divisor,
        };

        self.vertex_buffer_bindings
            .insert(binding_index, vertex_buffer_binding);
    }

    pub fn commit(&self, element_buffer_object: Option<Box<dyn OpenGLObject>>) {
        // Calculate total strides for various vertex attribute binding indexes.
        if let Some(max_binding_index) = self
            .vertex_attribs
            .iter()
            .max_by_key(|attrib| attrib.binding_index())
            .map(|attrib| attrib.binding_index())
        {
            let mut strides = vec![0u32; (max_binding_index + 1) as usize];

            for vertex_attrib in self.vertex_attribs.iter() {
                unsafe {
                    gl::EnableVertexArrayAttrib(self.handle(), vertex_attrib.index());
                    vertex_attrib.commit_vao_format(self.handle());
                    gl::VertexArrayAttribBinding(
                        self.handle(),
                        vertex_attrib.index(),
                        vertex_attrib.binding_index(),
                    );
                }

                strides[vertex_attrib.binding_index() as usize] += vertex_attrib.stride();
            }

            // Commit the VBO bindings.
            for (binding_index, binding) in self.vertex_buffer_bindings.iter() {
                unsafe {
                    gl::VertexArrayVertexBuffer(
                        self.handle(),
                        *binding_index,
                        binding.handle,
                        binding.vertex_offset,
                        strides[*binding_index as usize] as i32,
                    );

                    if binding.divisor != 0 {
                        gl::VertexArrayBindingDivisor(
                            self.handle(),
                            *binding_index,
                            binding.divisor,
                        );
                    }
                }
            }

            if let Some(ebo) = element_buffer_object {
                unsafe { gl::VertexArrayElementBuffer(self.handle(), ebo.handle()) };
            }
        }
    }
}

impl OpenGLObject for VertexArrayObject {
    fn handle(&self) -> u32 {
        self.handle
    }
}
