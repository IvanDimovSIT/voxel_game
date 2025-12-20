use bincode::{Decode, Encode};

/// the maximum number of variants the voxel enum can have,
/// used for performance optimisations
pub const MAX_VOXEL_VARIANTS: usize = 32;

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
    WaterSource,
    WaterDown,
    Water1,
    Water2,
    Water3,
    Water4,
    StoneBrick,
    StonePillar,
    Snow,
    Ice,
    Bomb,
    Glass,
}
impl Voxel {
    /// voxels that are fully or partially transparent
    pub const TRANSPARENT: [Self; 9] = [
        Self::None,
        Self::Glass,
        Self::WaterSource,
        Self::WaterDown,
        Self::Water1,
        Self::Water2,
        Self::Water3,
        Self::Water4,
        Self::Ice,
    ];

    /// voxels that can fall down
    pub const FALLING: [Self; 4] = [Self::Sand, Self::Dirt, Self::Grass, Self::Snow];

    pub const WATER: [Self; 6] = [
        Self::WaterSource,
        Self::WaterDown,
        Self::Water1,
        Self::Water2,
        Self::Water3,
        Self::Water4,
    ];

    pub const NON_SOURCE_WATER: [Self; 5] = [
        Self::WaterDown,
        Self::Water1,
        Self::Water2,
        Self::Water3,
        Self::Water4,
    ];

    pub const PARTIAL_HEIGHT: [Self; 4] = [Self::Water1, Self::Water2, Self::Water3, Self::Water4];
    pub const HALF_SIZE: f32 = 0.5;

    #[inline(always)]
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
            Self::WaterSource => "Water",
            Self::WaterDown => "Water (Flowing down)",
            Self::Water1 => "Water (Level 1)",
            Self::Water2 => "Water (Level 2)",
            Self::Water3 => "Water (Level 3)",
            Self::Water4 => "Water (Level 4)",
            Self::StoneBrick => "Stone Brick",
            Self::StonePillar => "Stone Pillar",
            Self::Snow => "Snow",
            Self::Ice => "Ice",
            Self::Bomb => "Bomb",
        }
    }

    pub fn is_solid(self) -> bool {
        !matches!(
            self,
            Voxel::None
                | Voxel::WaterSource
                | Voxel::WaterDown
                | Voxel::Water1
                | Voxel::Water2
                | Voxel::Water3
                | Voxel::Water4
        )
    }
}
impl Default for Voxel {
    fn default() -> Self {
        Self::None
    }
}
