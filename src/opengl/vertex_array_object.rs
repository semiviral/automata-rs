use super::OpenGLObject;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
pub enum VertexFormat {
    F32(bool),
    F64,
    U8,
    U16,
    U32,
}

impl VertexFormat {
    pub const fn get_stride(&self) -> u32 {
        match self {
            Self::F32(_) | Self::U32 => 4,
            Self::F64 => 8,
            Self::U8 => 1,
            Self::U16 => 2,
        }
    }

    pub const fn as_u32(self) -> u32 {
        match self {
            Self::F32(_) => gl::FLOAT,
            Self::F64 => gl::DOUBLE,
            Self::U8 => gl::UNSIGNED_BYTE,
            Self::U16 => gl::UNSIGNED_SHORT,
            Self::U32 => gl::UNSIGNED_INT,
        }
    }
}

struct VertexAttribute {
    index: u32,
    dimensions: i32,
    offset: u32,
    binding_index: u32,
    format: VertexFormat,
}

impl VertexAttribute {
    fn commit_vao_format(&self, vao_handle: u32) {
        unsafe {
            match self.format {
                VertexFormat::F32(normalized) => {
                    gl::VertexArrayAttribFormat(
                        vao_handle,
                        self.index,
                        self.dimensions,
                        self.format.as_u32(),
                        normalized as u8,
                        self.offset,
                    );
                }
                VertexFormat::F64 => {
                    gl::VertexArrayAttribLFormat(
                        vao_handle,
                        self.index,
                        self.dimensions,
                        self.format.as_u32(),
                        self.offset,
                    );
                }
                VertexFormat::U8 | VertexFormat::U16 | VertexFormat::U32 => {
                    gl::VertexArrayAttribIFormat(
                        vao_handle,
                        self.index,
                        self.dimensions,
                        self.format.as_u32(),
                        self.offset,
                    );
                }
            }
        }
    }
}

struct VertexBufferObjectBinding {
    handle: u32,
    vertex_offset: isize,
    divisor: u32,
}

pub struct VertexArrayObject {
    handle: u32,
    vertex_attribs: Vec<VertexAttribute>,
    vertex_buffer_bindings: BTreeMap<u32, VertexBufferObjectBinding>,
}

impl VertexArrayObject {
    pub fn new() -> Self {
        let mut handle = 0;
        unsafe { gl::CreateVertexArrays(1, &raw mut handle) };

        Self {
            handle,
            vertex_attribs: Vec::new(),
            vertex_buffer_bindings: BTreeMap::new(),
        }
    }

    pub fn clear_vertex_attributes(&mut self) {
        self.vertex_attribs.clear();
    }

    pub fn allocate_vertex_attribute(
        &mut self,
        index: u32,
        dimensions: u32,
        offset: u32,
        binding_index: u32,
        format: VertexFormat,
    ) {
        self.vertex_attribs.push(VertexAttribute {
            index,
            dimensions: dimensions as i32,
            offset,
            binding_index,
            format,
        });
    }

    pub fn allocate_vertex_buffer_binding(
        &mut self,
        binding_index: u32,
        buffer: &impl OpenGLObject,
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

    pub fn commit(&self, element_buffer_object: Option<&dyn OpenGLObject>) {
        // Calculate total strides for various vertex attribute binding indexes.
        if let Some(max_binding_index) = self
            .vertex_attribs
            .iter()
            .max_by_key(|attrib| attrib.binding_index)
            .map(|attrib| attrib.binding_index)
        {
            let mut strides = vec![0u32; (max_binding_index + 1) as usize];

            for vertex_attrib in self.vertex_attribs.iter() {
                unsafe {
                    gl::EnableVertexArrayAttrib(self.handle(), vertex_attrib.index);
                    vertex_attrib.commit_vao_format(self.handle());
                    gl::VertexArrayAttribBinding(
                        self.handle(),
                        vertex_attrib.index,
                        vertex_attrib.binding_index,
                    );
                }

                strides[vertex_attrib.binding_index as usize] +=
                    (vertex_attrib.dimensions as u32) * vertex_attrib.format.get_stride();
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

    pub fn bind(&self) {
        unsafe { gl::BindVertexArray(self.handle()) };
    }
}

impl OpenGLObject for VertexArrayObject {
    fn handle(&self) -> u32 {
        self.handle
    }
}
