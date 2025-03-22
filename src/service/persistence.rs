use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use macroquad::logging::{error, info, warn};

use crate::{
    model::area::{Area, AreaLocation},
    service::area_generation::generate_area,
};

const SERIALIZATION_CONFIG: Configuration = config::standard();

fn get_filepath(area_x: u32, area_y: u32, world_name: &str) -> String {
    format!("{world_name}/area{area_x}_{area_y}.dat")
}

pub fn store(area: &Area, world_name: &str) {
    let filepath = get_filepath(area.get_x(), area.get_y(), world_name);

    let encode_result = match encode_to_vec(area, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding area: {}", err);
            return;
        }
    };

    let _ = fs::create_dir_all(world_name);
    let mut file = match File::create(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error creating file '{}': {}", filepath, err);
            return;
        }
    };

    if let Err(err) = file.write_all(&encode_result) {
        error!("Error saving area data: {}", err)
    } else {
        info!("Saved '{}'", filepath)
    }
}

pub fn load(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, world_name);

    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't open file '{}': {}", filepath, err);
            return generate_area(area_location);
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!("Error reading file '{}': {}", filepath, err);
        return generate_area(area_location);
    };

    let (area, _read): (Area, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error decoding file '{}': {}", filepath, err);
            return generate_area(area_location);
        }
    };
    info!("Loaded '{}'", filepath);

    area
}