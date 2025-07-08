use std::hash::{DefaultHasher, Hash, Hasher};

use macroquad::logging::info;

use crate::{
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
        location::InternalLocation,
    },
    service::area_generation::{
        biome_type::BiomeTypeGenerator,
        cave_generator::CaveGenerator,
        terrain_type::TerrainTypeGenerator,
        trees::{TreeType, generate_trees, should_generate_tree},
        voxel_type_generator::VoxelTypeGenerator,
    },
    utils::StackVec,
};

const AREA_SURFACE: usize = (AREA_SIZE * AREA_SIZE) as usize;

fn hash_world_name(world_name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    world_name.hash(&mut hasher);
    hasher.finish()
}

pub struct AreaGenerator {
    seed: u64,
    height_noise: TerrainTypeGenerator,
    biome_type_noise: BiomeTypeGenerator,
    cave_noise: CaveGenerator,
    voxel_type_generator: VoxelTypeGenerator,
    tree_locations: StackVec<(InternalLocation, TreeType), AREA_SURFACE>,
}
impl AreaGenerator {
    /// generates an area at a specific location with the world name as the seed
    pub fn generate_area(area_location: AreaLocation, world_name: &str) -> Area {
        info!("Generating area: {:?}", area_location);
        let mut generator = AreaGenerator::new(world_name);

        let mut area = Area::new(area_location);
        for x in 0..AREA_SIZE {
            for y in 0..AREA_SIZE {
                generator.generate_column(&mut area, area_location, x, y);
            }
        }
        generate_trees(&mut area, &generator.tree_locations);

        debug_assert!(area.has_changed);
        area
    }

    /// generates a single column in an area and marks any potential tree locations
    fn generate_column(&mut self, area: &mut Area, area_location: AreaLocation, x: u32, y: u32) {
        let height = self.height_noise.sample(area_location, x, y);
        let biome_type = self.biome_type_noise.sample(area_location, x, y);

        for z_inverted in 1..=height {
            if self
                .cave_noise
                .should_be_cave(area_location, x, y, z_inverted)
            {
                continue;
            }
            let current_voxel = self.voxel_type_generator.calculate_voxel_type(
                area_location,
                x,
                y,
                z_inverted,
                height,
                biome_type,
            );
            area.set(
                InternalLocation::new(x, y, AREA_HEIGHT - z_inverted),
                current_voxel,
            );
        }

        let local = InternalLocation::new(x, y, AREA_HEIGHT - height);
        let tree_type = should_generate_tree(area.get(local), self.seed, area_location, local);
        if tree_type != TreeType::None {
            self.tree_locations.push((local, tree_type));
        }
    }

    /// private constructor
    fn new(world_name: &str) -> Self {
        let seed = hash_world_name(world_name);
        Self {
            seed,
            height_noise: TerrainTypeGenerator::new(seed),
            biome_type_noise: BiomeTypeGenerator::new(seed),
            cave_noise: CaveGenerator::new(seed),
            voxel_type_generator: VoxelTypeGenerator::new(seed),
            tree_locations: StackVec::new(),
        }
    }
}
