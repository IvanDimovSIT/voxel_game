use std::{
    collections::HashSet,
    fs::{self, File, remove_dir_all},
    io::{Read, Write},
    mem::take,
    sync::{Arc, Mutex},
};

use bincode::{decode_from_slice, encode_to_vec};
use macroquad::logging::{error, info, warn};

use crate::{
    model::area::{Area, AreaDTO, AreaLocation},
    service::{
        area_generation::generator::generate_area, persistence::config::SERIALIZATION_CONFIG,
    },
    utils::Semaphore,
};

use super::config::CONCURRENT_FILE_IO_COUNT;

static STORE_SEMAPHORE: Semaphore = Semaphore::new(CONCURRENT_FILE_IO_COUNT);

fn get_filepath(area_x: u32, area_y: u32, world_name: &str) -> String {
    format!("{world_name}/area{area_x}_{area_y}.dat")
}

/// stores an area and blocks the main thread
pub fn store_blocking(area: Area, world_name: &str) {
    debug_assert!(area.has_changed);
    let filepath = get_filepath(area.get_x(), area.get_y(), world_name);

    let area_dto: AreaDTO = area.into();
    let encode_result = match encode_to_vec(area_dto, SERIALIZATION_CONFIG) {
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

/// stores an area on a background thread
pub fn store(area: Area, world_name: String) {
    debug_assert!(area.has_changed);
    rayon::spawn(move || {
        STORE_SEMAPHORE.acquire();
        store_blocking(area, &world_name);
        STORE_SEMAPHORE.release();
    });
}

/// loads an area from disk
pub fn load_blocking(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, world_name);

    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't open file '{}': {}", filepath, err);
            return generate_area(area_location, world_name);
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!("Error reading file '{}': {}", filepath, err);
        return generate_area(area_location, world_name);
    };

    let (area_dto, _read): (AreaDTO, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error decoding file '{}': {}", filepath, err);
            return generate_area(area_location, world_name);
        }
    };
    info!("Loaded '{}'", filepath);

    area_dto.into_area(area_location, false)
}

/// struct to load areas asynchronously
pub struct AreaLoader {
    semaphore: Arc<Semaphore>,
    to_load: Arc<Mutex<HashSet<AreaLocation>>>,
    loaded: Arc<Mutex<Vec<Area>>>,
}
impl AreaLoader {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(CONCURRENT_FILE_IO_COUNT)),
            to_load: Arc::new(Mutex::new(HashSet::new())),
            loaded: Arc::new(Mutex::new(vec![])),
        }
    }

    /// starts background threads to load areas from disk
    pub fn batch_load(&mut self, areas_to_load: &[AreaLocation], world_name: &str) {
        let to_load_lock = self.to_load.lock().unwrap();
        let areas_to_load = areas_to_load
            .iter()
            .filter(|area_location| !to_load_lock.contains(area_location))
            .copied()
            .collect::<Vec<_>>();
        drop(to_load_lock);

        for area_to_load in areas_to_load {
            let semaphore = self.semaphore.clone();
            let to_load = self.to_load.clone();
            let loaded = self.loaded.clone();
            let world_name_owned = world_name.to_owned();
            rayon::spawn(move || {
                semaphore.acquire();
                let area = load_blocking(area_to_load, &world_name_owned);
                semaphore.release();
                let mut to_load_lock = to_load.lock().unwrap();
                let mut loaded_lock = loaded.lock().unwrap();
                to_load_lock.remove(&area.get_area_location());
                loaded_lock.push(area);
            });
        }
    }

    /// returns loaded areas
    pub fn get_loaded(&mut self) -> Vec<Area> {
        let areas = take(self.loaded.lock().unwrap().as_mut());

        areas
    }
}

/// deletes all world files and directory
pub fn delete_world(world_name: &str) {
    let is_path_invalid =
        world_name.contains(".") || world_name.contains("/") || world_name.contains("\\");

    if is_path_invalid {
        return;
    }

    if let Err(err) = remove_dir_all(world_name) {
        error!("Error deleting world: '{}'", err);
    } else {
        info!("Deleted world '{}'", world_name);
    }
}
