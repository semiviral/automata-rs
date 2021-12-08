use specs::{Component, VecStorage};

pub struct Pos(f32);
impl Component for Pos {
    type Storage = VecStorage<Self>;
}

pub struct InputSensitive;
impl Component for InputSensitive {
    type Storage = VecStorage<Self>;
}
