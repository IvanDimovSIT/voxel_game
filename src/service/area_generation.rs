use macroquad::logging::info;

use crate::model::area::{Area, AreaLocation};

pub fn generate_area(area_location: AreaLocation) -> Area {
    info!("Generating area: {:?}", area_location);
    //TODO: add area generation logic
    Area::new(area_location)
}
