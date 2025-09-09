use std::hash::{DefaultHasher, Hash, Hasher};

use macroquad::logging::info;

use crate::{
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, Area},
        location::{AreaLocation, InternalLocation},
    },
    service::area_generation::{
        biome_type::{BiomeType, BiomeTypeGenerator},
        cave_generator::CaveGenerator,
        lake_generator::LakeGenerator,
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

pub struct ColumnSamples {
    pub terrain_height: u32,
    pub is_cave_zone: bool,
    pub lake_depth: u32,
    pub biome_type: BiomeType,
}

pub struct AreaGenerator {
    seed: u64,
    height_noise: TerrainTypeGenerator,
    biome_type_noise: BiomeTypeGenerator,
    cave_noise: CaveGenerator,
    voxel_type_generator: VoxelTypeGenerator,
    lake_generator: LakeGenerator,
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

        area.update_all_column_heights();
        debug_assert!(area.has_changed);
        area
    }

    /// generates a single column in an area and marks any potential tree locations
    fn generate_column(&mut self, area: &mut Area, area_location: AreaLocation, x: u32, y: u32) {
        let column_sample = self.sample_column_characteristics(area_location, x, y);

        for z_inverted in 1..=column_sample.terrain_height {
            let current_voxel = self.voxel_type_generator.calculate_voxel_type(
                area_location,
                x,
                y,
                z_inverted,
                &column_sample,
            );
            let lake_voxel =
                LakeGenerator::generate_voxel(&column_sample, z_inverted).unwrap_or(current_voxel);
            if self.cave_noise.should_be_cave(
                &column_sample,
                lake_voxel,
                area_location,
                x,
                y,
                z_inverted,
            ) {
                continue;
            }

            area.set_without_updating_max_height(
                InternalLocation::new(x, y, AREA_HEIGHT - z_inverted),
                lake_voxel,
            );
        }

        let local = InternalLocation::new(x, y, AREA_HEIGHT - column_sample.terrain_height);
        let tree_type = should_generate_tree(area.get(local), self.seed, area_location, local);
        if tree_type != TreeType::None {
            self.tree_locations.push((local, tree_type));
        }
    }

    /// samples shared characteristics for the whole column
    fn sample_column_characteristics(
        &self,
        area_location: AreaLocation,
        x: u32,
        y: u32,
    ) -> ColumnSamples {
        let terrain_height = self.height_noise.sample(area_location, x, y);
        let is_cave_zone = self.cave_noise.is_cave_zone(area_location, x, y);
        let lake_depth = self.lake_generator.sample_lake_depth(
            is_cave_zone,
            terrain_height,
            area_location,
            x,
            y,
        );
        let biome_type = self.biome_type_noise.sample(area_location, x, y);

        ColumnSamples {
            terrain_height,
            is_cave_zone,
            lake_depth,
            biome_type,
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
            lake_generator: LakeGenerator::new(seed),
            tree_locations: StackVec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::model::voxel::Voxel;

    use super::*;

    #[test]
    fn test_generate_area() {
        let area = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test");
        assert!(area.has_changed);
        assert_eq!(area.get_x(), 123);
        assert_eq!(area.get_y(), 456);
        for x in 0..AREA_SIZE {
            for y in 0..AREA_SIZE {
                let mut max_height = None;
                let mut contains_stone = false;
                for z in 0..AREA_HEIGHT {
                    let voxel = area.get(InternalLocation::new(x, y, z));
                    if !Voxel::TRANSPARENT.contains(&voxel) && max_height.is_none() {
                        max_height = Some(z as u8);
                    }
                    if voxel == Voxel::Stone {
                        contains_stone = true;
                    }
                }
                assert!(max_height.is_some());
                assert!(contains_stone);
                assert_eq!(max_height.unwrap(), area.sample_height(x, y));
            }
        }
    }

    #[test]
    fn test_generate_area_different_locations() {
        let area1 = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test");
        let area2 = AreaGenerator::generate_area(AreaLocation::new(999, 400), "test");
        assert!(check_if_areas_are_different(&area1, &area2));
    }

    #[test]
    fn test_generate_area_different_seeds() {
        let area1 = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test1");
        let area2 = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test2");
        assert!(check_if_areas_are_different(&area1, &area2));
    }

    #[test]
    fn test_generate_same_area() {
        let area1 = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test");
        let area2 = AreaGenerator::generate_area(AreaLocation::new(123, 456), "test");
        assert!(!check_if_areas_are_different(&area1, &area2));
    }

    #[test]
    fn test_genearate_area_heights_calculated_correctly() {
        let areas: Vec<_> = (0..10)
            .into_iter()
            .map(|x| AreaGenerator::generate_area(AreaLocation::new(x, 123), "test"))
            .collect();

        let mut areas_calculated_heights = areas.clone();
        for area in &mut areas_calculated_heights {
            area.update_all_column_heights();
        }

        let are_all_identical = areas
            .iter()
            .zip(areas_calculated_heights.iter())
            .all(|(a1, a2)| !check_if_areas_are_different(a1, a2));

        assert!(are_all_identical);
    }

    fn check_if_areas_are_different(area1: &Area, area2: &Area) -> bool {
        for x in 0..AREA_SIZE {
            for y in 0..AREA_SIZE {
                let height1 = area1.sample_height(x, y);
                let height2 = area2.sample_height(x, y);

                if height1 != height2 {
                    return true;
                }

                for z in 0..AREA_HEIGHT {
                    let loc = InternalLocation::new(x, y, z);
                    let voxel1 = area1.get(loc);
                    let voxel2 = area2.get(loc);
                    if voxel1 != voxel2 {
                        return true;
                    }
                }
            }
        }

        return false;
    }
}
