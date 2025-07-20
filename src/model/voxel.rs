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

    pub fn display_name(self) -> &'static str {
        match self {
            Voxel::None => "None",
            Voxel::Cobblestone => "Cobblestone",
            Voxel::Sand => "Sand",
            Voxel::Grass => "Grass",
            Voxel::Wood => "Wood",
            Voxel::Leaves => "Leaves",
            Voxel::Brick => "Brick",
            Voxel::Dirt => "Dirt",
            Voxel::Boards => "Wooden Boards",
            Voxel::Stone => "Stone",
            Voxel::Clay => "Clay",
            Voxel::Lamp => "Lamp",
            Voxel::Trampoline => "Trampoline",
        }
    }
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
