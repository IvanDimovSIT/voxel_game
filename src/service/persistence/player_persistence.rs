use std::fs::create_dir_all;

use crate::{
    model::player_info::{PlayerInfo, PlayerInfoDTO},
    service::persistence::generic_persistence::{read_binary_object, write_binary_object},
};

fn get_filepath(world_name: &str) -> String {
    format!("{world_name}/player.dat")
}

pub fn load_player_info(world_name: &str) -> Option<PlayerInfo> {
    let filepath = get_filepath(world_name);
    let player_info_dto: Option<PlayerInfoDTO> = read_binary_object(&filepath);
    player_info_dto.map(|dto| dto.into())
}

pub fn save_player_info(world_name: &str, player_info: &PlayerInfo) {
    let dto = player_info.create_dto();
    let filepath = get_filepath(world_name);
    let _ = create_dir_all(world_name);
    let _result = write_binary_object(&filepath, &dto);
}
