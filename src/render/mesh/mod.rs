mod multi_draw_indirect;
mod vertex_array;

pub use multi_draw_indirect::*;
pub use vertex_array::*;

pub struct QuadIndexes<T> {
    indexes: [T; 6],
}

impl<T> QuadIndexes<T> {
    pub const fn new(indexes: [T; 6]) -> Self {
        Self { indexes }
    }
}

impl<T> std::ops::Index<usize> for QuadIndexes<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.indexes[index]
    }
}

impl<T> std::ops::IndexMut<usize> for QuadIndexes<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.indexes[index]
    }
}

pub struct QuadVertexes<T> {
    vertexes: [T; 4],
}

impl<T> QuadVertexes<T> {
    pub const fn new(vertexes: [T; 4]) -> Self {
        Self { vertexes }
    }
}

impl<T> std::ops::Index<usize> for QuadVertexes<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vertexes[index]
    }
}

impl<T> std::ops::IndexMut<usize> for QuadVertexes<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.vertexes[index]
    }
}

#[derive(Debug)]
pub struct PackedVertex {
    pub xyz: i32,
    pub uvz: i32,
}
