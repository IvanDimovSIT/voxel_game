use crate::model::area::{Area, AreaLocation};


pub fn generate_area(area_location: AreaLocation) -> Area {
    println!("Generating area: {area_location:?}");
    //TODO: add area generation logic
    Area::new(area_location)
}