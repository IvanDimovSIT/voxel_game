use bincode::{Decode, Encode};

/// the maximum number of variants the voxel enum can have,
/// used for performance optimisations
pub const MAX_VOXEL_VARIANTS: usize = 100;

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq, Hash)]
pub enum Voxel {
    None,
    Cobblestone,
    Sand,
    Grass,
    Wood,
    Leaves,
    Brick,
    Dirt,
    Boards,
    Stone,
    Clay,
    Lamp,
    Trampoline,
    Cactus,
    Glass,
}
impl Voxel {
    /// voxels that are fully or partially transparent
    pub const TRANSPARENT: [Self; 2] = [Self::None, Self::Glass];

    /// voxels that can fall down
    pub const FALLING: [Self; 3] = [Self::Sand, Self::Dirt, Self::Grass];

    pub fn index(self) -> usize {
        let index = self as usize;
        debug_assert!(
            index < MAX_VOXEL_VARIANTS,
            "number of voxel variants exceeds the maximum allowed"
        );
        index
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Cobblestone => "Cobblestone",
            Self::Sand => "Sand",
            Self::Grass => "Grass",
            Self::Wood => "Wood",
            Self::Leaves => "Leaves",
            Self::Brick => "Brick",
            Self::Dirt => "Dirt",
            Self::Boards => "Wooden Boards",
            Self::Stone => "Stone",
            Self::Clay => "Clay",
            Self::Lamp => "Lamp",
            Self::Trampoline => "Trampoline",
            Self::Glass => "Glass",
            Self::Cactus => "Cactus",
        }
    }
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
