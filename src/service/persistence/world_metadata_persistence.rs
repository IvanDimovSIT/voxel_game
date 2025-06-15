use std::{
    fs::{self, File},
    io::{Read, Write},
};

use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use macroquad::prelude::{error, info, warn};

use crate::service::{persistence::config::SERIALIZATION_CONFIG, world_time::WorldTime};

#[derive(Debug, Clone, Encode, Decode)]
pub struct WorldMetadata {
    pub delta: f32,
}
impl WorldMetadata {
    pub fn new(world_time: &WorldTime) -> Self {
        Self {
            delta: world_time.get_delta(),
        }
    }
}

fn get_metadata_filepath(world_name: &str) -> String {
    format!("{world_name}/world.dat")
}

/// stores the world time
pub fn store_world_metadata(world_metadata: WorldMetadata, world_name: &str) {
    let filepath = get_metadata_filepath(world_name);

    let encode_result = match encode_to_vec(world_metadata, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding world metadata: {}", err);
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
        error!("Error saving world metadata: {}", err)
    } else {
        info!("Saved '{}'", filepath)
    }
}

/// loads the world metadata from disk
pub fn load_world_metadata(world_name: &str) -> Option<WorldMetadata> {
    let filepath = get_metadata_filepath(world_name);

    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't open file '{}': {}", filepath, err);
            return None;
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!("Error reading file '{}': {}", filepath, err);
        return None;
    };

    let (metadata, _read): (WorldMetadata, usize) =
        match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
            Ok(ok) => ok,
            Err(err) => {
                error!("Error decoding file '{}': {}", filepath, err);
                return None;
            }
        };
    info!("Loaded '{}'", filepath);

    Some(metadata)
}
