use std::{
    error::Error, fs::File, io::{Read, Write}
};

use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use macroquad::logging::{error, info, warn};
use zip::{write::{FileOptions, SimpleFileOptions}, ZipArchive, ZipWriter};

use crate::{
    model::area::{Area, AreaLocation},
    service::area_generation::generate_area,
};

const SERIALIZATION_CONFIG: Configuration = config::standard();


fn file_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755)
}

fn get_filepath(area_x: u32, area_y: u32) -> String {
    format!("area{}_{}.dat", area_x, area_y)
}

fn get_zip_path(world_name: &str) -> String {
    format!("{world_name}.dat")
}

fn open_or_create(zip_path: &str) -> Result<File, ()> {
    match File::open(zip_path) {
        Ok(ok) => Ok(ok),
        Err(err) => {
            warn!("Can't open '{}': '{}', creating instead", zip_path, err);
            File::create(zip_path)
                .map_err(|create_err| {
                    error!("Error creating '{}': '{}'", zip_path, create_err);
                    ()
                })
        },
    }
}


pub fn store(area: &Area, world_name: &str) {
    let zip_path = get_zip_path(world_name);
    let zip = if let Ok(ok) = open_or_create(&zip_path) {
        ok
    } else {
        return;
    };

    let filepath = get_filepath(area.get_x(), area.get_y());

    let encode_result = match encode_to_vec(area, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding area: {}", err);
            return;
        }
    };
    let mut zip_writer = ZipWriter::new(zip);
    

    if let Err(err) = zip_writer.start_file(&filepath, file_options()) {
        error!("Error starting file write to zip: {}", err);
        return;
    }

    if let Err(err) = zip_writer.write_all(&encode_result) {
        error!("Error writing encoded data to zip file: {}", err);
        return;
    }

    if let Err(err) = zip_writer.finish() {
        error!("Error finishing zip write: {}", err);
        return;
    }

    info!("Stored '{}/{}'", zip_path, filepath);
}

pub fn load(area_location: AreaLocation, world_name: &str) -> Area {
    let zip_path = get_zip_path(world_name);
    let filepath = get_filepath(area_location.x, area_location.y);

    let zip_file = match File::open(&zip_path) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't open file '{}': {}", zip_path, err);
            return generate_area(area_location);
        }
    };
    let mut zip = match ZipArchive::new(zip_file) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Count open zip archive: {}", err);
            return generate_area(area_location);
        },
    };

    let mut area_file = match zip.by_name(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't find '{}': {}", filepath, err);
            return generate_area(area_location);
        },
    };


    let mut buf = vec![];
    if let Err(err) = area_file.read_to_end(&mut buf) {
        warn!("Error reading area file: {}", err);
        return generate_area(area_location);
    }

    let (area, _read): (Area, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error decoding file '{}': {}", filepath, err);
            return generate_area(area_location);
        }
    };
    info!("Loaded '{}/{}'", zip_path, filepath);

    area
}
