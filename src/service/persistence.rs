use std::{fs::File, io::{Read, Write}};

use bincode::{config::{self, Configuration}, decode_from_slice, encode_to_vec};

use crate::{model::area::{Area, AreaLocation}, service::area_generation::generate_area};


const SAVE_PATH: &str = "./";
const SERIALIZATION_CONFIG: Configuration = config::standard();

pub fn store(area: &Area) {
    let filepath = format!("{SAVE_PATH}{}_{}.dat", area.get_x(), area.get_y());

    let encode_result = match encode_to_vec(area, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error encoding area: {err}");
            return;
        },
    };
    
    let mut file = match File::create(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error creating file '{filepath}': {err}");
            return;
        },
    };

    if let Err(err) = file.write_all(&encode_result) {
        println!("Error saving area data: {err}")
    }
} 

pub fn load(area_location: AreaLocation) -> Area {
    let filepath = format!("{SAVE_PATH}{}_{}.dat", area_location.x, area_location.y);

    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error oppening file '{filepath}': {err}");
            return generate_area(area_location);
        },
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        println!("Error reading file '{filepath}': {err}");
        return generate_area(area_location);
    };

    let (area, _read): (Area, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error decoding file '{filepath}': {err}");
        return generate_area(area_location);
        },
    };
    
    area
}
