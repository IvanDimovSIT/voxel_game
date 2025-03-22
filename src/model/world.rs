use std::collections::HashMap;

use macroquad::prelude::{error, warn};

use crate::{
    model::{
        area::{AREA_HEIGHT, Area},
        location::Location,
        voxel::Voxel,
    },
    service::persistence::{self, batch_load},
};

use super::{
    area::{AREA_SIZE, AreaLocation},
    location::InternalLocation,
};

pub struct World {
    world_name: String,
    areas: HashMap<AreaLocation, Area>,
}
impl World {
    pub fn new(world_name: impl Into<String>) -> Self {
        Self {
            world_name: world_name.into(),
            areas: HashMap::new(),
        }
    }

    pub fn load_area(&mut self, area_location: AreaLocation) {
        if self.areas.contains_key(&area_location) {
            return;
        }
        let area = persistence::load_blocking(area_location, &self.world_name);
        self.areas.insert(area_location, area);
    }

    pub fn unload_area(&mut self, area_location: AreaLocation) {
        if !self.areas.contains_key(&area_location) {
            return;
        }
        if let Some(unloaded) = self.areas.remove(&area_location) {
            persistence::store(unloaded, self.world_name.clone());
        } else {
            error!("Missing loaded {:?}", area_location);
        }
    }

    pub fn convert_global_to_local_location(location: InternalLocation) -> InternalLocation {
        let local_x = location.x % AREA_SIZE;
        let local_y = location.y % AREA_SIZE;

        InternalLocation::new(local_x, local_y, location.z)
    }

    pub fn convert_global_to_area_location(location: InternalLocation) -> AreaLocation {
        let area_x = location.x / AREA_SIZE;
        let area_y = location.y / AREA_SIZE;

        AreaLocation::new(area_x, area_y)
    }

    pub fn convert_global_to_area_and_local_location(
        location: InternalLocation,
    ) -> (AreaLocation, InternalLocation) {
        (
            Self::convert_global_to_area_location(location),
            Self::convert_global_to_local_location(location),
        )
    }

    pub fn get_renderable_voxels_for_area(
        &mut self,
        area_location: AreaLocation,
    ) -> Vec<(InternalLocation, Voxel)> {
        self.load_area(area_location);
        let area = match self.areas.get(&area_location) {
            Some(ok) => ok,
            None => {
                error!("Area {:?} not loaded", area_location);
                return vec![];
            }
        };
        let x_offset = area_location.x * AREA_SIZE;
        let y_offset = area_location.y * AREA_SIZE;

        let mut result = vec![];

        for z in 0..AREA_HEIGHT {
            for y in 0..AREA_SIZE {
                for x in 0..AREA_SIZE {
                    let current_location = InternalLocation::new(x, y, z);
                    let voxel = area.get(current_location);
                    if voxel == Voxel::None {
                        continue;
                    }
                    if area.has_nonempty_neighbours(current_location) {
                        continue;
                    }

                    result.push((InternalLocation::new(x + x_offset, y + y_offset, z), voxel));
                }
            }
        }

        result
    }

    pub fn get(&mut self, location: InternalLocation) -> Voxel {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location);
        self.load_area(area_location);
        let area = &self.areas[&area_location];
        area.get(local_location)
    }

    pub fn get_without_loading(&self, location: InternalLocation) -> Option<Voxel> {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location);
        self.areas
            .get(&area_location)
            .map(|area| area.get(local_location))
    }

    pub fn set(&mut self, location: InternalLocation, voxel: Voxel) {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location);
        self.load_area(area_location);
        let area = self.areas.get_mut(&area_location).expect("Area not loaded");
        area.set(local_location, voxel);
    }

    pub fn retain_areas(&mut self, area_locations: &[AreaLocation]) {
        let area_locations_to_load = area_locations
            .iter()
            .filter(|location| !self.areas.contains_key(location))
            .map(|reference| *reference)
            .collect();

        let loaded_areas = batch_load(area_locations_to_load, self.world_name.clone());
        for area in loaded_areas {
            let area_location = AreaLocation::new(area.get_x(), area.get_y());
            self.areas.insert(area_location, area);
        }

        let areas_to_unload: Vec<_> = self
            .areas
            .keys()
            .filter(|loaded| !area_locations.contains(&loaded))
            .map(|x| *x)
            .collect();

        for area_location in areas_to_unload {
            self.unload_area(area_location);
        }
    }
}
