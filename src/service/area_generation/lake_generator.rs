use libnoise::{Fbm, Generator, Simplex};

use crate::{
    model::{area::AREA_HEIGHT, location::AreaLocation, voxel::Voxel},
    service::area_generation::{
        algorithms::{get_point_on_noise_map, normalise_sample},
        biome_type::BiomeType,
        generator::ColumnSamples,
    },
};

const MAX_TERRAIN_HEIGHT: u32 = (AREA_HEIGHT as f32 * 0.32) as u32;
const LAKE_MAX_Z_INVERTED: u32 = (AREA_HEIGHT as f32 * 0.27) as u32;
const MIN_LAKE_HEIGHT: i32 = 2;
const MIN_LAKE_NOISE: i32 = 70;

pub struct LakeGenerator {
    lake_noise: Fbm<2, Simplex<2>>,
}
impl LakeGenerator {
    pub fn new(seed: u64) -> Self {
        let lake_noise = Simplex::new(seed.wrapping_add(10)).fbm(2, 0.004, 1.3, 0.35);

        Self { lake_noise }
    }

    pub fn sample_lake_depth(
        &self,
        is_cave_zone: bool,
        terrain_height: u32,
        area_location: AreaLocation,
        x: u32,
        y: u32,
    ) -> u32 {
        if is_cave_zone
            || terrain_height > MAX_TERRAIN_HEIGHT
            || terrain_height <= LAKE_MAX_Z_INVERTED
        {
            return 0;
        }

        let point2d = get_point_on_noise_map(area_location, x, y);
        let height_value =
            normalise_sample(self.lake_noise.sample(point2d)) as i32 - MIN_LAKE_NOISE;
        let lake_depth = height_value / 3;

        if lake_depth < MIN_LAKE_HEIGHT {
            return 0;
        }

        lake_depth as u32
    }

    pub fn generate_voxel(column_samples: &ColumnSamples, z_inverted: u32) -> Option<Voxel> {
        if column_samples.lake_depth == 0 {
            return None;
        }

        if z_inverted > LAKE_MAX_Z_INVERTED {
            return Some(Voxel::None);
        }

        let min_water = LAKE_MAX_Z_INVERTED - column_samples.lake_depth;

        if z_inverted >= min_water {
            if column_samples.biome_type == BiomeType::Cold && z_inverted == LAKE_MAX_Z_INVERTED {
                Some(Voxel::Ice)
            } else {
                Some(Voxel::WaterSource)
            }
        } else {
            None
        }
    }
}
