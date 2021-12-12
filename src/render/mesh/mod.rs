mod multi_draw_indirect;

pub use multi_draw_indirect::*;

pub trait Mesh {
    // const LAYER: RenderLayer;

    fn visible(&self) -> bool;
    fn draw(&mut self);
}
