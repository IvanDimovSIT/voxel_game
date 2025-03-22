use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use macroquad::logging::{error, info, warn};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    spawn,
};

use crate::{
    model::area::{Area, AreaLocation},
    service::area_generation::generate_area,
    utils::Semaphore,
};

const SERIALIZATION_CONFIG: Configuration = config::standard();
const CONCURRENT_FILE_IO_COUNT: usize = 16;
static STORE_SEMAPHORE: Semaphore = Semaphore::new(CONCURRENT_FILE_IO_COUNT);

fn get_filepath(area_x: u32, area_y: u32, world_name: &str) -> String {
    format!("{world_name}/area{area_x}_{area_y}.dat")
}

fn store_blocking(area: Area, world_name: String) {
    let filepath = get_filepath(area.get_x(), area.get_y(), &world_name);

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

pub fn store(area: Area, world_name: String) {
    spawn(move || {
        STORE_SEMAPHORE.acquire();
        store_blocking(area, world_name);
        STORE_SEMAPHORE.release();
    });
}

pub fn load_blocking(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, &world_name);

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

pub fn batch_load(area_locations: Vec<AreaLocation>, world_name: String) -> Vec<Area> {
    area_locations
        .chunks(CONCURRENT_FILE_IO_COUNT)
        .map(|chunk| {
            chunk
                .par_iter()
                .map(|area_location| {
                    let world_name_ref = &world_name;
                    load_blocking(area_location.clone(), world_name_ref)
                })
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect()
}
