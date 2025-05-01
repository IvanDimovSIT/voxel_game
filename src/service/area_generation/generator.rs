use std::hash::{DefaultHasher, Hash, Hasher};

use macroquad::logging::info;

use crate::{
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
        location::InternalLocation,
        voxel::Voxel,
    },
    service::area_generation::{
        biome_type::BiomeTypeGenerator,
        terrain_type::TerrainTypeGenerator,
        trees::{TreeType, generate_trees, should_generate_tree},
    },
    utils::StackVec,
};

use super::biome_type::BiomeType;

const STONE_HEIGHT: u32 = 30;

fn hash_world_name(world_name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    world_name.hash(&mut hasher);
    hasher.finish()
}

fn calculate_voxel_type(z: u32, height: u32, biome_type: BiomeType) -> Voxel {
    match biome_type {
        BiomeType::Dry => {
            if height >= STONE_HEIGHT {
                return Voxel::Stone;
            }

            if (z + 2) >= height {
                return Voxel::Sand;
            }

            Voxel::Stone
        }
        BiomeType::Wet => {
            if z >= height {
                return Voxel::Grass;
            }

            if z + 1 >= height {
                return Voxel::Dirt;
            }

            Voxel::Stone
        }
    }
}

/// generates an area at a specific location with the world name as the seed
pub fn generate_area(area_location: AreaLocation, world_name: &str) -> Area {
    info!("Generating area: {:?}", area_location);

    let seed = hash_world_name(world_name);
    let height_noise = TerrainTypeGenerator::new(seed);
    let type_noise = BiomeTypeGenerator::new(seed);
    const AREA_SURFACE: usize = (AREA_SIZE * AREA_SIZE) as usize;
    let mut trees_location: StackVec<(InternalLocation, TreeType), AREA_SURFACE> = StackVec::new();

    let mut area = Area::new(area_location);
    for x in 0..AREA_SIZE {
        for y in 0..AREA_SIZE {
            let height = height_noise.sample(area_location, x, y);
            let biome_type = type_noise.sample(area_location, x, y);
            for z in 1..=height as u32 {
                let current_voxel = calculate_voxel_type(z, height, biome_type);
                area.set(InternalLocation::new(x, y, AREA_HEIGHT - z), current_voxel);
            }
            let local = InternalLocation::new(x, y, AREA_HEIGHT - height);
            let tree_type = should_generate_tree(area.get(local), seed, area_location, local);
            if tree_type != TreeType::None {
                trees_location.push((local, tree_type));
            }
        }
    }
    generate_trees(&mut area, &trees_location);

    debug_assert!(area.has_changed);
    area
}
