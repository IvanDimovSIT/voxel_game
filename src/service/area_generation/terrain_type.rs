use libnoise::{Fbm, Generator, Max, Simplex};

use crate::{
    model::area::{AREA_HEIGHT, AreaLocation},
    service::area_generation::sample_noise_map::get_point_on_noise_map,
};

pub struct TerrainTypeGenerator {
    noise: Max<2, Fbm<2, Simplex<2>>, Fbm<2, Simplex<2>>>,
}
impl TerrainTypeGenerator {
    const HEIGHT_SCALE: f64 = 0.5;

    pub fn new(seed: u64) -> Self {
        let noise1 = Simplex::new(seed).fbm(6, 0.008, 1.8, 0.45);
        let noise2 = Simplex::new(seed ^ u64::MAX).fbm(4, 0.004, 1.8, 0.45);

        Self {
            noise: noise1.max(noise2),
        }
    }

    pub fn sample(&self, area_location: AreaLocation, x: u32, y: u32) -> u32 {
        let point = get_point_on_noise_map(area_location, x, y);
        let value = self.noise.sample(point);

        ((value * Self::HEIGHT_SCALE * AREA_HEIGHT as f64) as u32) + AREA_HEIGHT / 4
    }
}
