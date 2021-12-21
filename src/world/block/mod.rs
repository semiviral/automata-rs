mod block_registry;

pub use block_registry::*;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    id: u16,
    color: u16,
    light_lvl: u8,
}

impl Block {
    pub const AIR: Self = Self::new(0, 0, 0);

    pub const fn new(id: u16, color: u16, light_lvl: u8) -> Self {
        Self {
            id,
            color,
            light_lvl,
        }
    }

    pub const fn id(&self) -> u16 {
        self.id
    }

    pub const fn color(&self) -> u16 {
        self.color
    }

    pub const fn light_lvl(&self) -> u8 {
        self.light_lvl
    }
}

impl Eq for Block {}
impl PartialEq for Block {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Ord for Block {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}
impl PartialOrd for Block {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}
