use std::collections::HashMap;

use macroquad::prelude::error;

use crate::{
    model::{
        area::{AREA_HEIGHT, Area},
        location::Location,
        voxel::Voxel,
    },
    service::persistence,
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
        let area = persistence::load(area_location, &self.world_name);
        self.areas.insert(area_location, area);
    }

    pub fn unload_area(&mut self, area_location: AreaLocation) {
        if !self.areas.contains_key(&area_location) {
            return;
        }
        persistence::store(&self.areas[&area_location], &self.world_name);
        self.areas.remove(&area_location);
    }

    pub fn convert_global_to_local_location(
        location: InternalLocation,
    ) -> InternalLocation {
        let local_x = location.x % AREA_SIZE;
        let local_y = location.y % AREA_SIZE;

        InternalLocation::new(local_x, local_y, location.z)
    }

    pub fn convert_global_to_area_location(
        location: InternalLocation,
    ) -> AreaLocation {
        let area_x = location.x / AREA_SIZE;
        let area_y = location.y / AREA_SIZE;

        AreaLocation::new(area_x, area_y)
    }

    pub fn convert_global_to_area_and_local_location(location: InternalLocation) -> (AreaLocation, InternalLocation) {
        (
            Self::convert_global_to_area_location(location),
            Self::convert_global_to_local_location(location)
        )
    }

    pub fn get_renderable_voxels_for_area(
        &self,
        area_location: AreaLocation,
    ) -> Vec<(InternalLocation, Voxel)> {
        let area = match self.areas.get(&area_location) {
            Some(ok) => ok,
            None => {
                error!("Area {area_location:?} not loaded");
                return vec![];
            }
        };
        let xy_offset = area_location.x * AREA_SIZE;

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

                    result.push((
                        InternalLocation::new(x + xy_offset, y + xy_offset, z),
                        voxel,
                    ));
                }
            }
        }

        result
    }

    pub fn get(&mut self, location: InternalLocation) -> Voxel {
        let (area_location, local_location) = Self::convert_global_to_area_and_local_location(location);
        self.load_area(area_location);
        let area = &self.areas[&area_location];
        area.get(local_location)
    }

    pub fn get_without_loading(&self, location: InternalLocation) -> Option<Voxel> {
        let (area_location, local_location) = Self::convert_global_to_area_and_local_location(location);
        self.areas
            .get(&area_location)
            .map(|area| area.get(local_location))
    }

    pub fn set(&mut self, location: InternalLocation, voxel: Voxel) {
        let (area_location, local_location) = Self::convert_global_to_area_and_local_location(location);
        self.load_area(area_location);
        let area = self.areas.get_mut(&area_location).expect("Area not loaded");
        area.set(local_location, voxel);
    }
}
