use macroquad::{logging::info, rand::rand};

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
            for z in 1..=(2+rand()%6) {
                area.set(
                    InternalLocation::new(x, y, AREA_HEIGHT - z),
                    crate::model::voxel::Voxel::Stone,
                );
            }
        }
    }

    debug_assert!(area.has_changed);
    area
}
