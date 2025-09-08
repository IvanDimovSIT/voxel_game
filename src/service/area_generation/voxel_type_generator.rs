use libnoise::{Fbm, Generator, Simplex};

use crate::{
    model::{
        area::{AREA_HEIGHT, AreaLocation},
        voxel::Voxel,
    },
    service::area_generation::{
        algorithms::{get_point_on_noise_map_3d, normalise_sample},
        biome_type::BiomeType,
        generator::ColumnSamples,
    },
};

const CLAY_THRESHOLD: f64 = 70.0;
const BASE_STONE_THRESHOLD: f64 = 120.0;
const SAND_HEIGHT: u32 = 3;

pub struct VoxelTypeGenerator {
    alternative_voxel_type: Fbm<3, Simplex<3>>,
}
impl VoxelTypeGenerator {
    pub fn new(seed: u64) -> Self {
        let seed = seed.wrapping_add(1);
        Self {
            alternative_voxel_type: Simplex::new(seed).fbm(2, 0.07, 2.0, 0.3),
        }
    }

    pub fn calculate_voxel_type(
        &self,
        area_location: AreaLocation,
        x: u32,
        y: u32,
        z_inverted: u32,
        column_samples: &ColumnSamples,
    ) -> Voxel {
        match column_samples.biome_type {
            BiomeType::Dry => self.calculate_voxel_type_for_dry_biome(
                area_location,
                x,
                y,
                z_inverted,
                column_samples.terrain_height,
            ),
            BiomeType::Wet => self.calculate_voxel_type_for_wet_biome(
                area_location,
                x,
                y,
                z_inverted,
                column_samples.terrain_height,
            ),
        }
    }

    fn calculate_voxel_type_for_dry_biome(
        &self,
        area_location: AreaLocation,
        x: u32,
        y: u32,
        z_inverted: u32,
        height: u32,
    ) -> Voxel {
        if (z_inverted + SAND_HEIGHT) >= height {
            let threshold = (1.0 - z_inverted as f64 / AREA_HEIGHT as f64) * BASE_STONE_THRESHOLD;
            if self.should_generate_alternative_voxel(area_location, x, y, z_inverted, threshold) {
                return Voxel::Stone;
            } else {
                return Voxel::Sand;
            }
        }

        Voxel::Stone
    }

    fn calculate_voxel_type_for_wet_biome(
        &self,
        area_location: AreaLocation,
        x: u32,
        y: u32,
        z_inverted: u32,
        height: u32,
    ) -> Voxel {
        if z_inverted >= height {
            if self.should_generate_alternative_voxel(
                area_location,
                x,
                y,
                z_inverted,
                CLAY_THRESHOLD,
            ) {
                return Voxel::Clay;
            } else {
                return Voxel::Grass;
            }
        }

        if z_inverted + 1 >= height {
            if self.should_generate_alternative_voxel(
                area_location,
                x,
                y,
                z_inverted,
                CLAY_THRESHOLD,
            ) {
                return Voxel::Clay;
            } else {
                return Voxel::Dirt;
            }
        }

        Voxel::Stone
    }

    fn should_generate_alternative_voxel(
        &self,
        area_location: AreaLocation,
        x: u32,
        y: u32,
        z_inverted: u32,
        threshold: f64,
    ) -> bool {
        let point = get_point_on_noise_map_3d(area_location, x, y, z_inverted);
        let value = normalise_sample(self.alternative_voxel_type.sample(point));

        value >= threshold
    }
}
