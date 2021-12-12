mod multidraw_indirect;

pub use multidraw_indirect::*;

pub trait Mesh {
    // const LAYER: RenderLayer;

    fn visible(&self) -> bool;
    fn draw(&self);
}
