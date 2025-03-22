use macroquad::logging::info;

use crate::model::{
    area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
    location::InternalLocation,
};

pub fn generate_area(area_location: AreaLocation) -> Area {
    //TODO: Improve area generation
    info!("Generating area: {:?}", area_location);
    let mut area = Area::new(area_location);
    for x in 0..AREA_SIZE {
        for y in 0..AREA_SIZE {
            area.set(
                InternalLocation::new(x, y, AREA_HEIGHT - 1),
                crate::model::voxel::Voxel::Stone,
            );
        }
    }

    area
}
