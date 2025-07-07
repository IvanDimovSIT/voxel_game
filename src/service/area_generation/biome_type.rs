use libnoise::{Fbm, Generator, Simplex};

use crate::{
    model::area::AreaLocation,
    service::area_generation::algorithms::{get_point_on_noise_map, normalise_sample},
};

#[derive(Debug, Clone, Copy)]
pub enum BiomeType {
    Dry,
    Wet,
}

pub struct BiomeTypeGenerator {
    noise: Fbm<2, Simplex<2>>,
}
impl BiomeTypeGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            noise: Simplex::new(seed).fbm(2, 0.002, 1.5, 0.3),
        }
    }

    pub fn sample(&self, area_location: AreaLocation, x: u32, y: u32) -> BiomeType {
        let point = get_point_on_noise_map(area_location, x, y);
        let value = normalise_sample(self.noise.sample(point)) as i32;

        match value {
            0..70 => BiomeType::Wet,
            _ => BiomeType::Dry,
        }
    }
}
