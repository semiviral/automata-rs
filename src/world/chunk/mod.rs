mod mesher;

pub use mesher::*;

use crate::collections::Palette;

use super::block::Block;

pub const CHUNK_SIZE: i32 = 32;
pub const CHUNK_SIZE_SQUARED: i32 = CHUNK_SIZE.pow(2);
pub const CHUNK_SIZE_CUBED: i32 = CHUNK_SIZE.pow(3);
pub const CHUNK_SIZE_SHIFT: i32 = CHUNK_SIZE.trailing_zeros() as i32;
pub const CHUNK_SIZE_MASK: i32 = CHUNK_SIZE - 1;

pub trait ChunkGenerationStep {
    fn gen_pass(&self, blocks: &mut Palette<Block>);
}

pub struct ChunkGenerationSystem {
}