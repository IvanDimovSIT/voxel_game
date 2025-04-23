use std::{
    fs::File,
    io::{Read, Write},
};

use bincode::{decode_from_slice, encode_to_vec};
use macroquad::prelude::{error, info};

use super::config::SERIALIZATION_CONFIG;

const WORLD_LIST_FILEPATH: &str = "worlds.dat";

pub fn read_world_list() -> Vec<String> {
    let mut file = match File::open(&WORLD_LIST_FILEPATH) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                "Couldn't open world list file '{}': {}",
                WORLD_LIST_FILEPATH, err
            );
            return vec![];
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!(
            "Error reading world list file '{}': {}",
            WORLD_LIST_FILEPATH, err
        );
        return vec![];
    };

    let (list, _read): (Vec<String>, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                "Error decoding world list file '{}': {}",
                WORLD_LIST_FILEPATH, err
            );
            return vec![];
        }
    };
    info!("Loaded world list: {:?}", list);

    list
}

pub fn write_world_list(list: &[String]) {
    let encode_result = match encode_to_vec(list, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding world list: {}", err);
            return;
        }
    };

    let mut file = match File::create(WORLD_LIST_FILEPATH) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error creating file '{}': {}", WORLD_LIST_FILEPATH, err);
            return;
        }
    };

    if let Err(err) = file.write_all(&encode_result) {
        error!("Error saving world list: {}", err)
    } else {
        info!("Saved world list '{}'", WORLD_LIST_FILEPATH)
    }
}
