use std::{
    hash::{DefaultHasher, Hash, Hasher},
    u64,
};

use libnoise::{Generator, Simplex};
use macroquad::logging::info;

use crate::model::{
    area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
    location::InternalLocation,
    voxel::Voxel,
};

fn hash_world_name(world_name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    world_name.hash(&mut hasher);
    hasher.finish()
}

fn get_height_noise(seed: u64) -> impl Generator<2> {
    let noise1 = Simplex::new(seed).fbm(6, 0.008, 1.8, 0.45);

    let noise2 = Simplex::new(seed ^ u64::MAX).fbm(4, 0.004, 1.8, 0.45);

    noise1.max(noise2)
}

fn get_voxel_type_noise(seed: u64) -> impl Generator<2> {
    Simplex::new(seed).fbm(2, 0.002, 1.5, 0.3)
}

fn get_point_on_noise_map(area_location: AreaLocation, x: u32, y: u32) -> [f64; 2] {
    [
        (x + area_location.x * AREA_SIZE) as f64,
        (y + area_location.y * AREA_SIZE) as f64,
    ]
}

fn calculate_height(
    height_noise: &impl Generator<2>,
    area_location: AreaLocation,
    x: u32,
    y: u32,
) -> u32 {
    const HEIGHT_SCALE: f64 = 0.3;
    let noise_value = height_noise.sample(get_point_on_noise_map(area_location, x, y));

    ((noise_value * HEIGHT_SCALE * AREA_HEIGHT as f64) as u32) + AREA_HEIGHT / 4
}

fn calculate_voxel_type(
    type_noise: &impl Generator<2>,
    area_location: AreaLocation,
    x: u32,
    y: u32,
) -> Voxel {
    let noise_value = type_noise.sample(get_point_on_noise_map(area_location, x, y));

    match (noise_value * 100.0) as u64 {
        0..60 => Voxel::Grass,
        60..70 => Voxel::Stone,
        _ => Voxel::Sand,
    }
}

pub fn generate_area(area_location: AreaLocation, world_name: &str) -> Area {
    info!("Generating area: {:?}", area_location);

    let seed = hash_world_name(world_name);
    let height_noise = get_height_noise(seed);
    let type_noise = get_voxel_type_noise(seed);

    let mut area = Area::new(area_location);
    for x in 0..AREA_SIZE {
        for y in 0..AREA_SIZE {
            let height = calculate_height(&height_noise, area_location, x, y);
            let voxel = calculate_voxel_type(&type_noise, area_location, x, y);
            for z in 1..=height as u32 {
                let current_voxel = if z + 1 >= height { voxel } else { Voxel::Stone };
                area.set(InternalLocation::new(x, y, AREA_HEIGHT - z), current_voxel);
            }
        }
    }

    debug_assert!(area.has_changed);
    area
}
