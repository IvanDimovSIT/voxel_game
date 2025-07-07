use crate::model::{
    area::{AREA_SIZE, AreaLocation},
    location::InternalLocation,
};

pub fn get_point_on_noise_map(area_location: AreaLocation, x: u32, y: u32) -> [f64; 2] {
    [
        (x + area_location.x * AREA_SIZE) as f64,
        (y + area_location.y * AREA_SIZE) as f64,
    ]
}

pub fn get_point_on_noise_map_3d(area_location: AreaLocation, x: u32, y: u32, z: u32) -> [f64; 3] {
    [
        (x + area_location.x * AREA_SIZE) as f64,
        (y + area_location.y * AREA_SIZE) as f64,
        z as f64,
    ]
}

pub fn combine_seed(seed: u64, area_location: AreaLocation, local: InternalLocation) -> u64 {
    let mut combined_seed = seed;

    combined_seed = combined_seed.wrapping_mul(0x517cc1b727220a95);

    combined_seed = combined_seed.wrapping_add(area_location.x as u64);
    combined_seed = combined_seed.rotate_left(11);
    combined_seed = combined_seed.wrapping_add(area_location.y as u64);
    combined_seed = combined_seed.rotate_left(13);

    combined_seed = combined_seed.wrapping_add(local.x as u64);
    combined_seed = combined_seed.rotate_left(17);
    combined_seed = combined_seed.wrapping_add(local.y as u64);
    combined_seed = combined_seed.rotate_left(19);
    combined_seed = combined_seed.wrapping_add(local.z as u64);
    combined_seed = combined_seed.rotate_left(23);

    combined_seed = combined_seed.wrapping_mul(0xff51afd7ed558ccd);
    combined_seed
}

/// SplitMix64 algorithm
pub fn split_mix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// normalises a noise sample value from the range [-1.0, 1.0] to [0.0, 100.0]
pub fn normalise_sample(value: f64) -> f64 {
    (value + 1.0) * 50.0
}
