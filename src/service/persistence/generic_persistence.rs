use std::{
    any::type_name,
    fs::{File, create_dir, create_dir_all, remove_dir_all},
    io::{Read, Write},
};

use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use macroquad::prelude::{error, info};

use crate::service::persistence::{
    config::{BASE_SAVE_PATH, SERIALIZATION_CONFIG},
    world_persistence::get_world_directory,
};

pub fn read_binary_object<T: Decode<()>>(filepath: &str) -> Option<T> {
    let filepath = format!("{BASE_SAVE_PATH}{filepath}");
    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                "Couldn't open {} file '{}': {}",
                type_name::<T>(),
                filepath,
                err
            );
            return None;
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!(
            "Error reading {} file '{}': {}",
            type_name::<T>(),
            filepath,
            err
        );
        return None;
    };

    let (object, _read): (T, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!(
                "Error decoding {} file '{}': {}",
                type_name::<T>(),
                filepath,
                err
            );
            return None;
        }
    };
    info!("Loaded {}: {}", type_name::<T>(), filepath);

    Some(object)
}

pub fn write_binary_object<T: Encode>(filepath: &str, object: &T) -> Result<(), ()> {
    let filepath = format!("{BASE_SAVE_PATH}{filepath}");
    let encode_result = match encode_to_vec(object, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding {}: {}", type_name::<T>(), err);
            return Err(());
        }
    };

    let mut file = match File::create(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error creating file '{}': {}", filepath, err);
            return Err(());
        }
    };

    if let Err(err) = file.write_all(&encode_result) {
        error!("Error saving {}: {}", type_name::<T>(), err);
        Err(())
    } else {
        info!("Saved {} to '{}'", type_name::<T>(), filepath);
        Ok(())
    }
}

pub fn initialise_save_directory() {
    if let Err(err) = create_dir(BASE_SAVE_PATH) {
        error!(
            "Error creating save directory '{}': {}",
            BASE_SAVE_PATH, err
        );
    } else {
        info!("Save directory '{}' initialised", BASE_SAVE_PATH);
    }
}

pub fn create_directory(world_name: &str) -> Result<(), std::io::Error> {
    create_dir_all(get_world_directory(world_name))
}

pub fn remove_directory(world_name: &str) -> Result<(), std::io::Error> {
    remove_dir_all(get_world_directory(world_name))
}
