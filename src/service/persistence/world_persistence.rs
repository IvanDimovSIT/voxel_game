use std::{
    collections::HashSet,
    fs::{self, remove_dir_all},
    mem::take,
    sync::{Arc, Mutex},
};

use macroquad::logging::{error, info};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    model::area::{Area, AreaDTO, AreaLocation},
    service::{
        area_generation::generator::AreaGenerator,
        persistence::generic_persistence::{read_binary_object, write_binary_object},
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
    let _ = fs::create_dir_all(world_name);
    let _result = write_binary_object(&filepath, &area_dto);
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

/// stores all areas and blocks the main thread
pub fn store_all_blocking(areas: Vec<Area>, world_name: String) {
    areas.into_par_iter().for_each(|area| {
        STORE_SEMAPHORE.acquire();
        store_blocking(area, &world_name);
        STORE_SEMAPHORE.release();
    });
}

/// loads an area from disk
pub fn load_blocking(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, world_name);
    let area_dto: Option<AreaDTO> = read_binary_object(&filepath);

    area_dto
        .map(|dto| dto.into_area(area_location, false))
        .unwrap_or_else(|| AreaGenerator::generate_area(area_location, world_name))
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

    /// loads areas in parallel and blocks the main thread until finished
    pub fn load_all_blocking(
        &mut self,
        areas_to_load: &[AreaLocation],
        world_name: &str,
    ) -> Vec<Area> {
        areas_to_load
            .par_iter()
            .map(|area_location| {
                self.semaphore.acquire();
                let loaded_area = load_blocking(*area_location, world_name);
                self.semaphore.release();
                loaded_area
            })
            .collect()
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
