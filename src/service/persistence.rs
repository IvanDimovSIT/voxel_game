use std::{
    collections::HashSet, fs::{self, File}, io::{Read, Write}, mem::take, sync::{Arc, Mutex}
};

use bincode::{
    config::{self, Configuration},
    decode_from_slice, encode_to_vec,
};
use macroquad::logging::{error, info, warn};

use crate::{
    model::area::{Area, AreaLocation},
    service::area_generation::generate_area,
    utils::Semaphore,
};

const SERIALIZATION_CONFIG: Configuration = config::standard();
const CONCURRENT_FILE_IO_COUNT: usize = 16;
static STORE_SEMAPHORE: Semaphore = Semaphore::new(CONCURRENT_FILE_IO_COUNT);

fn get_filepath(area_x: u32, area_y: u32, world_name: &str) -> String {
    format!("{world_name}/area{area_x}_{area_y}.dat")
}

pub fn store_blocking(area: &Area, world_name: &str) {
    let filepath = get_filepath(area.get_x(), area.get_y(), &world_name);

    let encode_result = match encode_to_vec(area, SERIALIZATION_CONFIG) {
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

pub fn store(area: Area, world_name: String) {
    rayon::spawn(move || {
        STORE_SEMAPHORE.acquire();
        store_blocking(&area, &world_name);
        STORE_SEMAPHORE.release();
    });
}

pub fn load_blocking(area_location: AreaLocation, world_name: &str) -> Area {
    let filepath = get_filepath(area_location.x, area_location.y, &world_name);

    let mut file = match File::open(&filepath) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Couldn't open file '{}': {}", filepath, err);
            return generate_area(area_location);
        }
    };

    let mut buf = vec![];
    if let Err(err) = file.read_to_end(&mut buf) {
        error!("Error reading file '{}': {}", filepath, err);
        return generate_area(area_location);
    };

    let (area, _read): (Area, usize) = match decode_from_slice(&buf, SERIALIZATION_CONFIG) {
        Ok(ok) => ok,
        Err(err) => {
            error!("Error decoding file '{}': {}", filepath, err);
            return generate_area(area_location);
        }
    };
    info!("Loaded '{}'", filepath);

    area
}

pub struct AreaLoader {
    semaphore: Arc<Semaphore>,
    to_load: Arc<Mutex<HashSet<AreaLocation>>>,
    loaded: Arc<Mutex<Vec<Area>>>
} 
impl AreaLoader {
    pub fn new() -> Self {
        Self { 
            semaphore: Arc::new(Semaphore::new(CONCURRENT_FILE_IO_COUNT)), 
            to_load: Arc::new(Mutex::new(HashSet::new())), 
            loaded: Arc::new(Mutex::new(vec![]))
        }
    }

    pub fn batch_load(&mut self, areas_to_load: &[AreaLocation], world_name: &str) {
        let to_load_lock = self.to_load.lock().unwrap();
        let areas_to_load = areas_to_load.iter()
            .filter(|area_location| !to_load_lock.contains(area_location))
            .map(|loc| *loc)
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

    pub fn get_loaded(&mut self) -> Vec<Area> {
        let areas = take(self.loaded.lock().unwrap().as_mut());

        areas
    }
}