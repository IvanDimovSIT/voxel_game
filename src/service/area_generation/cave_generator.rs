use libnoise::{Fbm, Generator, Max, Simplex};

use crate::{
    model::{
        area::{AREA_HEIGHT, AreaLocation},
        voxel::Voxel,
    },
    service::area_generation::{algorithms::normalise_sample, generator::ColumnSamples},
};

use super::algorithms::{get_point_on_noise_map, get_point_on_noise_map_3d};

const MIN_CAVES_THRESHOLD: i32 = 70;
const MAX_CAVES_THRESHOLD: i32 = 90;

/// min height to NOT generate caves
const MIN_HEIGHT: u32 = (AREA_HEIGHT as f32 * 0.2) as u32;

/// max height to NOT generate caves
const MAX_HEIGHT: u32 = (AREA_HEIGHT as f32 * 0.9) as u32;

const SHOULD_CHECK_FOR_CAVES_THRESHOLD: i32 = 80;

pub struct CaveGenerator {
    cave_noise: Max<3, Fbm<3, Simplex<3>>, Fbm<3, Simplex<3>>>,
    check_noise: Fbm<2, Simplex<2>>,
}
impl CaveGenerator {
    pub fn new(seed: u64) -> Self {
        let noise1 = Simplex::new(seed).fbm(6, 0.08, 1.3, 0.45);
        let noise2 = Simplex::new(seed ^ u64::MAX).fbm(8, 0.04, 1.2, 0.35);

        Self {
            cave_noise: noise1.max(noise2),
            check_noise: Simplex::new(seed).fbm(2, 0.003, 1.3, 0.3),
        }
    }

    pub fn is_cave_zone(&self, area_location: AreaLocation, x: u32, y: u32) -> bool {
        let point2d = get_point_on_noise_map(area_location, x, y);
        let check_value = normalise_sample(self.check_noise.sample(point2d)) as i32;

        check_value >= SHOULD_CHECK_FOR_CAVES_THRESHOLD
    }

    pub fn should_be_cave(
        &self,
        column_samples: &ColumnSamples,
        voxel: Voxel,
        area_location: AreaLocation,
        x: u32,
        y: u32,
        z: u32,
    ) -> bool {
        if !column_samples.is_cave_zone || voxel == Voxel::None || voxel == Voxel::WaterSource {
            return false;
        }

        if !(MIN_HEIGHT..MAX_HEIGHT).contains(&z) {
            return false;
        }

        let point3d = get_point_on_noise_map_3d(area_location, x, y, z);
        let value = normalise_sample(self.cave_noise.sample(point3d)) as i32;

        (MIN_CAVES_THRESHOLD..MAX_CAVES_THRESHOLD).contains(&value)
    }
}
