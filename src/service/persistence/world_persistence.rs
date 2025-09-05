use std::{
    collections::HashSet,
    mem::take,
    sync::{Arc, Mutex},
};

use macroquad::logging::{error, info};
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{
    model::area::{Area, AreaDTO, AreaLocation},
    service::{
        area_generation::generator::AreaGenerator,
        persistence::generic_persistence::{
            create_directory, read_binary_object, remove_directory, write_binary_object,
        },
    },
};

use super::config::BASE_SAVE_PATH;

const IS_COMPRESSED: bool = true;

fn get_filepath(area_x: u32, area_y: u32, world_name: &str) -> String {
    format!("{world_name}/area{area_x}_{area_y}.dat")
}

pub fn get_world_directory(world_name: &str) -> String {
    format!("{BASE_SAVE_PATH}{world_name}")
}

/// stores an area and blocks the main thread
pub fn store_blocking(area: Area, world_name: &str) {
    debug_assert!(area.has_changed);
    let filepath = get_filepath(area.get_x(), area.get_y(), world_name);
    let area_dto: AreaDTO = area.into();
    let _ = create_directory(world_name);
    let _result = write_binary_object(&filepath, &area_dto, IS_COMPRESSED);
}

/// stores an area on a background thread
pub fn store(area: Area, world_name: String) {
    debug_assert!(area.has_changed);
    rayon::spawn(move || {
        store_blocking(area, &world_name);
    });
}

/// stores all areas and blocks the main thread
pub fn store_all_blocking(areas: Vec<Area>, world_name: String) {
    areas.into_par_iter().for_each(|area| {
        store_blocking(area, &world_name);
    });
}

/// loads an area from disk
pub fn load_blocking(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, world_name);
    let area_dto: Option<AreaDTO> = read_binary_object(&filepath, IS_COMPRESSED);

    area_dto
        .map(|dto| dto.into_area(area_location, false))
        .unwrap_or_else(|| AreaGenerator::generate_area(area_location, world_name))
}

/// struct to load areas asynchronously
pub struct AreaLoader {
    to_load: Arc<Mutex<HashSet<AreaLocation>>>,
    loaded: Arc<Mutex<Vec<Area>>>,
}
impl AreaLoader {
    pub fn new() -> Self {
        Self {
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
            .map(|area_location| load_blocking(*area_location, world_name))
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
            let to_load = self.to_load.clone();
            let loaded = self.loaded.clone();
            let world_name_owned = world_name.to_owned();
            rayon::spawn(move || {
                let area = load_blocking(area_to_load, &world_name_owned);
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

    if let Err(err) = remove_directory(world_name) {
        error!("Error deleting world: '{}'", err);
    } else {
        info!("Deleted world '{}'", world_name);
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, fs::remove_dir_all, time::Instant};

    use crate::model::{
        area::{AREA_HEIGHT, AREA_SIZE},
        location::InternalLocation,
    };

    use super::*;

    struct TestWorldName {
        name: &'static str,
    }
    impl TestWorldName {
        fn new(name: &'static str) -> Self {
            Self { name }
        }
    }
    impl Drop for TestWorldName {
        fn drop(&mut self) {
            let dir_name = get_world_directory(self.name);
            remove_dir_all(&dir_name).unwrap();
        }
    }

    #[test]
    pub fn test_world_persistence_load() {
        let world_name = TestWorldName::new("test_world_persistence_load_temp_test_world");

        let area_location = AreaLocation::new(0, 0);
        let area = AreaGenerator::generate_area(area_location, world_name.name);
        store_blocking(area.clone(), world_name.name);

        let loaded_area = load_blocking(area_location, world_name.name);

        assert!(!loaded_area.has_changed);
        assert_eq!(loaded_area.get_x(), area_location.x);
        assert_eq!(loaded_area.get_y(), area_location.y);
        assert_areas_equal(&area, &loaded_area);
    }

    #[test]
    pub fn test_world_persistence_area_loader_batch_load() {
        let world_name = TestWorldName::new("test_world_persistence_area_loader_batch_load");

        let area_locations = [
            AreaLocation::new(0, 0),
            AreaLocation::new(1, 0),
            AreaLocation::new(0, 1),
            AreaLocation::new(1, 1),
        ];
        let mut areas: HashMap<_, _> = area_locations
            .into_iter()
            .map(|loc| (loc, AreaGenerator::generate_area(loc, world_name.name)))
            .collect();

        store_all_blocking(
            areas.clone().into_values().collect(),
            world_name.name.to_owned(),
        );

        let mut area_loader = AreaLoader::new();
        area_loader.batch_load(&area_locations, world_name.name);

        let start = Instant::now();
        loop {
            let loaded = area_loader.get_loaded();
            for area in loaded {
                assert!(!area.has_changed);
                let original = areas.remove(&area.get_area_location()).unwrap();
                assert_areas_equal(&area, &original);
            }
            let ellpased_ms = start.elapsed().as_millis();

            if ellpased_ms > 100 {
                break;
            }
        }

        assert!(areas.is_empty());
    }

    fn assert_areas_equal(area1: &Area, area2: &Area) {
        for z in 0..AREA_HEIGHT {
            for y in 0..AREA_SIZE {
                for x in 0..AREA_SIZE {
                    let local_location = InternalLocation::new(x, y, z);
                    assert_eq!(area1.get(local_location), area2.get(local_location));
                }
            }
        }
    }
}
