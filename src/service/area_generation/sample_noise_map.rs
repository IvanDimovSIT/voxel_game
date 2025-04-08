use crate::model::area::{AREA_SIZE, AreaLocation};

pub fn get_point_on_noise_map(area_location: AreaLocation, x: u32, y: u32) -> [f64; 2] {
    [
        (x + area_location.x * AREA_SIZE) as f64,
        (y + area_location.y * AREA_SIZE) as f64,
    ]
}
