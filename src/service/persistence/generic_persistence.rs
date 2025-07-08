use std::{
    any::type_name,
    fs::File,
    io::{Read, Write},
};

use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use macroquad::prelude::{error, info};

use crate::service::persistence::config::SERIALIZATION_CONFIG;

pub fn read_binary_object<T: Decode<()>>(filepath: &str) -> Option<T> {
    let mut file = match File::open(filepath) {
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
    let encode_result = match encode_to_vec(object, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding {}: {}", type_name::<T>(), err);
            return Err(());
        }
    };

    let mut file = match File::create(filepath) {
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
