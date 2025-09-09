use libnoise::{Fbm, Generator, Max, Simplex};

use crate::{
    model::{area::AREA_HEIGHT, location::AreaLocation},
    service::area_generation::algorithms::get_point_on_noise_map,
};

pub struct TerrainTypeGenerator {
    noise: Max<2, Fbm<2, Simplex<2>>, Fbm<2, Simplex<2>>>,
    height_modifier: Fbm<2, Simplex<2>>,
}
impl TerrainTypeGenerator {
    const HEIGHT_SCALE: f64 = 0.5;

    pub fn new(seed: u64) -> Self {
        let noise1 = Simplex::new(seed).fbm(6, 0.008, 1.8, 0.45);
        let noise2 = Simplex::new(seed ^ u64::MAX).fbm(4, 0.004, 1.8, 0.45);
        let height_modifier = Simplex::new(seed.wrapping_add(1)).fbm(2, 0.004, 0.004, 0.2);

        Self {
            noise: noise1.max(noise2),
            height_modifier,
        }
    }

    pub fn sample(&self, area_location: AreaLocation, x: u32, y: u32) -> u32 {
        let point = get_point_on_noise_map(area_location, x, y);
        let value = self.noise.sample(point).clamp(0.1, 1.0);
        let height_mod = 1.0 - (self.height_modifier.sample(point) + 1.0) / 2.0;

        ((height_mod * value * Self::HEIGHT_SCALE * AREA_HEIGHT as f64) as u32) + AREA_HEIGHT / 4
    }
}
