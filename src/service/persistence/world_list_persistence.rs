use crate::service::persistence::generic_persistence::{read_binary_object, write_binary_object};

const WORLD_LIST_FILEPATH: &str = "worlds.dat";

pub fn read_world_list() -> Vec<String> {
    read_binary_object(WORLD_LIST_FILEPATH).unwrap_or_default()
}

pub fn write_world_list(list: &Vec<String>) {
    let _result = write_binary_object(WORLD_LIST_FILEPATH, list);
}
