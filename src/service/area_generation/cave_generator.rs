use libnoise::{Fbm, Generator, Max, Simplex};

use crate::model::area::{AREA_HEIGHT, AreaLocation};

use super::algorithms::{get_point_on_noise_map, get_point_on_noise_map_3d};

const MIN_CAVES_THRESHOLD: i32 = 30;
const MAX_CAVES_THRESHOLD: i32 = 70;

/// min height to NOT generate caves
const MIN_HEIGHT: u32 = AREA_HEIGHT - 48;

/// max height to NOT generate caves
const MAX_HEIGHT: u32 = AREA_HEIGHT - 2;

const SHOULD_CHECK_FOR_CAVES_THRESHOLD: i32 = 60;

pub struct CaveGenerator {
    noise: Max<3, Fbm<3, Simplex<3>>, Fbm<3, Simplex<3>>>,
    check_noise: Fbm<2, Simplex<2>>,
}
impl CaveGenerator {
    pub fn new(seed: u64) -> Self {
        let noise1 = Simplex::new(seed).fbm(6, 0.08, 1.3, 0.45);
        let noise2 = Simplex::new(seed ^ u64::MAX).fbm(8, 0.04, 1.2, 0.35);

        Self {
            noise: noise1.max(noise2),
            check_noise: Simplex::new(seed).fbm(2, 0.003, 1.3, 0.3),
        }
    }

    pub fn should_be_cave(&self, area_location: AreaLocation, x: u32, y: u32, z: u32) -> bool {
        if !(MIN_HEIGHT..MAX_HEIGHT).contains(&z) {
            return false;
        }

        let point2d = get_point_on_noise_map(area_location, x, y);
        let check_value = (self.check_noise.sample(point2d) * 100.0) as i32;
        if check_value < SHOULD_CHECK_FOR_CAVES_THRESHOLD {
            return false;
        }

        let point3d = get_point_on_noise_map_3d(area_location, x, y, z);
        let value = (self.noise.sample(point3d) * 100.0) as i32;

        (MIN_CAVES_THRESHOLD..MAX_CAVES_THRESHOLD).contains(&value)
    }
}
