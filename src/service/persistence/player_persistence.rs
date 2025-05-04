use std::{
    fs::File,
    io::{Read, Write},
};

use bincode::{decode_from_slice, encode_to_vec};
use macroquad::prelude::{error, info};

use crate::model::player_info::{PlayerInfo, PlayerInfoDTO};

use super::config::SERIALIZATION_CONFIG;

fn get_filepath(world_name: &str) -> String {
    format!("{world_name}/player.dat")
}

pub fn load_player_info(world_name: &str) -> Result<PlayerInfo, ()> {
    let path = get_filepath(world_name);
    let mut file = match File::open(&path) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Couldn't open player info '{}': {}", path, err);
            return Err(());
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!("Error reading player info file '{}': {}", path, err);
        return Err(());
    };

    let (dto, _read): (PlayerInfoDTO, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error decoding player info file '{}': {}", path, err);
            return Err(());
        }
    };
    info!("Loaded player info: {:?}", dto);

    Ok(dto.into())
}

pub fn save_player_info(world_name: &str, player_info: &PlayerInfo) {
    let dto = player_info.create_dto();

    let encode_result = match encode_to_vec(dto, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error encoding player info: {}", err);
            return;
        }
    };
    let path = get_filepath(world_name);

    let mut file = match File::create(&path) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error creating file '{}': {}", path, err);
            return;
        }
    };

    if let Err(err) = file.write_all(&encode_result) {
        error!("Error saving player info: {}", err)
    } else {
        info!("Saved player info '{}'", path)
    }
}
