use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
pub enum Voxel {
    None,
    Stone,
    Sand,
    Grass,
    Wood,
    Leaves,
    Brick,
    Dirt,
    Boards,
}
impl Voxel {
    pub fn index(self) -> usize {
        self as usize
    }
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
