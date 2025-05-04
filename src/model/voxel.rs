use bincode::{Decode, Encode};

/// the maximum number of variants the voxel enum can have,
/// used for performance optimisations
pub const MAX_VOXEL_VARIANTS: usize = 100;

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
        let index = self as usize;
        debug_assert!(
            index < MAX_VOXEL_VARIANTS,
            "number of voxel variants exceeds the maximum allowed"
        );
        index
    }
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
