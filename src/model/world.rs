use std::{collections::HashMap, mem::take, time::Instant};

use macroquad::prelude::{error, info};

use crate::{
    model::{
        area::{AREA_HEIGHT, Area},
        location::AreaLocation,
        voxel::Voxel,
    },
    service::persistence::world_persistence::{self, AreaLoader},
};

use super::{area::AREA_SIZE, location::InternalLocation};

pub struct World {
    world_name: String,
    areas: HashMap<AreaLocation, Area>,
    area_loader: AreaLoader,
    empty_area: Area,
}
impl World {
    pub fn new(world_name: impl Into<String>) -> Self {
        Self {
            world_name: world_name.into(),
            areas: HashMap::new(),
            area_loader: AreaLoader::new(),
            empty_area: Area::new(AreaLocation::new(0, 0)),
        }
    }

    pub fn load_area(&mut self, area_location: AreaLocation) {
        if self.areas.contains_key(&area_location) {
            return;
        }
        let area = world_persistence::load_blocking(area_location, &self.world_name);
        self.areas.insert(area_location, area);
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

    /// may return an empty area if not loaded
    pub fn get_area_without_loading(&self, area_location: AreaLocation) -> &Area {
        self.areas.get(&area_location).unwrap_or_else(|| {
            error!("Trying to read an unloaded area. Returning empty area.");
            &self.empty_area
        })
    }

    /// temporarily takes ownership of an area to be used with `get_with_cache`
    pub fn with_cached_area<L, F, T>(&mut self, location: L, f: F) -> T
    where
        L: Into<AreaLocation>,
        F: FnOnce(&mut World, &Area) -> T,
    {
        let area_location = location.into();
        self.load_area(area_location);
        let area = self.areas.remove(&area_location).expect("Area not loaded");
        let result = f(self, &area);
        self.return_area(area);

        result
    }

    /// returns ownership of an area
    pub fn return_area(&mut self, area: Area) {
        self.areas.insert(area.get_area_location(), area);
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
                    if area.has_non_transparent_neighbours(current_location) {
                        continue;
                    }

                    result.push((InternalLocation::new(x + x_offset, y + y_offset, z), voxel));
                }
            }
        }

        result
    }

    pub fn get(&mut self, location: impl Into<InternalLocation>) -> Voxel {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location.into());
        self.load_area(area_location);
        let area = &self.areas[&area_location];
        area.get(local_location)
    }

    pub fn get_with_cache(
        &mut self,
        location: impl Into<InternalLocation>,
        cached_area: Option<&Area>,
    ) -> Voxel {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location.into());

        if let Some(area) = cached_area {
            if area.get_area_location() == area_location {
                return area.get(local_location);
            }
        }

        self.load_area(area_location);
        let area = &self.areas[&area_location];
        area.get(local_location)
    }

    pub fn get_without_loading(&self, location: impl Into<InternalLocation>) -> Option<Voxel> {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location.into());
        self.areas
            .get(&area_location)
            .map(|area| area.get(local_location))
    }

    pub fn set(&mut self, location: impl Into<InternalLocation>, voxel: Voxel) {
        let (area_location, local_location) =
            Self::convert_global_to_area_and_local_location(location.into());
        self.load_area(area_location);
        let area = self.areas.get_mut(&area_location).expect("Area not loaded");
        area.has_changed = true;
        area.set(local_location, voxel);
    }

    /// loads all areas at the input locations asynchronously and unloads
    /// all areas not at the input locations asynchronously
    /// moves any loaded areas into the main area map
    pub fn retain_areas(&mut self, area_locations: &[AreaLocation]) {
        let loaded = self.area_loader.get_loaded();
        for area in loaded {
            let area_location = area.get_area_location();
            if self.areas.contains_key(&area_location) {
                continue;
            }
            self.areas.insert(area_location, area);
        }

        let area_locations_to_load = area_locations
            .iter()
            .filter(|location| !self.areas.contains_key(location))
            .copied()
            .collect::<Vec<_>>();

        self.area_loader
            .batch_load(&area_locations_to_load, &self.world_name);

        let areas_to_unload: Vec<_> = self
            .areas
            .keys()
            .filter(|loaded| !area_locations.contains(loaded))
            .copied()
            .collect();

        self.unload_areas(&areas_to_unload);
    }

    fn unload_areas(&mut self, areas_to_unload: &[AreaLocation]) {
        let mut unloaded = Vec::with_capacity(32);
        for area_location in areas_to_unload {
            let area_option = self.areas.remove(area_location);
            if let Some(area) = area_option {
                if area.has_changed {
                    unloaded.push(area);
                }
            }
        }
        if unloaded.is_empty() {
            return;
        }

        world_persistence::store(unloaded, self.world_name.clone());
    }

    pub fn get_loaded_areas_count(&self) -> usize {
        self.areas.len()
    }

    /// loads all areas and blocks the main thread
    pub fn load_all_blocking(&mut self, areas_to_load: &[AreaLocation]) {
        let start = Instant::now();
        let filtered_unloaded: Vec<_> = areas_to_load
            .iter()
            .filter(|area_location| !self.areas.contains_key(area_location))
            .copied()
            .collect();
        info!("Loading {} areas", filtered_unloaded.len());
        let areas = self
            .area_loader
            .load_all_blocking(&filtered_unloaded, &self.world_name);
        for area in areas {
            self.areas.insert(area.get_area_location(), area);
        }
        let end = start.elapsed();
        info!("Loaded in {}ms", end.as_millis());
    }

    /// saves all areas and clears memory
    pub fn save_all_blocking(&mut self) {
        let start = Instant::now();
        info!("Saving world...");
        let areas = take(&mut self.areas)
            .into_values()
            .filter(|area| area.has_changed)
            .collect();
        world_persistence::store_all_blocking(areas, self.world_name.clone());
        let end = start.elapsed();
        info!("Saved in {}ms", end.as_millis());
    }

    pub fn get_world_name(&self) -> &str {
        &self.world_name
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use crate::{
        model::location::Location, service::persistence::world_persistence::get_world_directory,
    };

    use super::*;

    #[test]
    fn test_get_and_set() {
        let mut world = World::new("test_world_test_get_and_set");

        for i in 0..10 {
            let x = i * 200;
            world.set(Location::new(x, 10, 10), Voxel::Brick);
        }

        for i in 0..10 {
            let x = i * 200;
            assert_eq!(world.get(Location::new(x, 10, 10)), Voxel::Brick);
        }
    }

    #[test]
    fn test_get_same_location() {
        let mut world = World::new("test_world_test_get_same_location");

        let mut voxels = vec![];
        for i in 0..10 {
            let x = i * 200;
            voxels.push(world.get(Location::new(x, 10, 10)));
        }

        for i in 0..10 {
            let x = i * 200;
            assert_eq!(world.get(Location::new(x, 10, 10)), voxels[i as usize]);
        }
    }

    #[test]
    fn test_get_renderable_locations_for_area() {
        let mut world = World::new("test_world_test_get_renderable_locations_for_area");

        let renderable = world.get_renderable_voxels_for_area(AreaLocation::new(0, 0));

        for (loc, voxel) in renderable {
            assert_ne!(voxel, Voxel::None);
            assert!((0..AREA_SIZE).contains(&loc.x));
            assert!((0..AREA_SIZE).contains(&loc.y));
            assert!((0..AREA_HEIGHT).contains(&loc.z));
        }
    }

    #[test]
    fn test_retain_areas() {
        let world_name = "test_world_test_retain_areas";
        let remove_dir = get_world_directory(world_name);
        let mut world = World::new(world_name);
        let initial_areas = [AreaLocation::new(0, 0), AreaLocation::new(1, 0)];

        world.retain_areas(&initial_areas);
        std::thread::sleep(Duration::from_millis(100));
        world.retain_areas(&initial_areas);

        assert_eq!(world.areas.len(), initial_areas.len());
        for area in initial_areas {
            assert!(world.areas.contains_key(&area));
        }

        let other_areas = [AreaLocation::new(0, 0), AreaLocation::new(2, 0)];

        world.retain_areas(&other_areas);
        std::thread::sleep(Duration::from_millis(100));
        world.retain_areas(&other_areas);

        assert_eq!(world.areas.len(), other_areas.len());
        for area in other_areas {
            assert!(world.areas.contains_key(&area));
        }

        fs::remove_dir_all(&remove_dir).unwrap();
    }

    #[test]
    fn test_load_all_blocking() {
        let world_name = "test_world_test_load_all_blocking";
        let mut world = World::new(world_name);
        let areas = [AreaLocation::new(0, 0), AreaLocation::new(1, 0)];

        world.load_all_blocking(&areas);

        assert_eq!(world.areas.len(), areas.len());
        for area in areas {
            assert!(world.areas.contains_key(&area));
        }
    }

    #[test]
    fn test_with_cached_area() {
        let world_name = "test_world_test_with_cached_area";
        let mut world = World::new(world_name);
        let area_location = AreaLocation::new(0, 0);
        let mut area = Area::new(AreaLocation::new(0, 0));
        let loc = InternalLocation::new(1, 2, 3);
        area.set(InternalLocation::new(1, 2, 3), Voxel::Brick);

        world.return_area(area.clone());
        assert_eq!(world.areas.len(), 1);

        world.with_cached_area(area_location, |world, contained_area| {
            assert!(world.areas.is_empty());
            assert_eq!(area.get(loc), contained_area.get(loc));
        });
        assert_eq!(world.areas.len(), 1);
    }
}
