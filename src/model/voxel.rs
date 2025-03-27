use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
pub enum Voxel {
    None,
    Stone,
    Sand,
    Grass,
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
