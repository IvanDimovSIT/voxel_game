use bincode::{Decode, Encode};

use crate::graphics::mesh_generator::MeshGenerator;

use super::{
    location::{InternalLocation, Location},
    voxel::Voxel,
    world::World,
};

pub const AREA_SIZE: u32 = 16;
pub const AREA_HEIGHT: u32 = 128;
pub const VOXELS_IN_AREA: usize = (AREA_SIZE * AREA_SIZE * AREA_HEIGHT) as usize;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AreaLocation {
    pub x: u32,
    pub y: u32,
}
impl AreaLocation {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}
impl From<InternalLocation> for AreaLocation {
    fn from(value: InternalLocation) -> Self {
        World::convert_global_to_area_location(value)
    }
}
impl From<Location> for AreaLocation {
    fn from(value: Location) -> Self {
        World::convert_global_to_area_location(value.into())
    }
}

#[derive(Debug, Clone)]
pub struct Area {
    pub has_changed: bool,
    area_location: AreaLocation,
    voxels: Box<[Voxel]>,
    max_height: Box<[u8]>,
}
impl Area {
    pub fn new(area_location: AreaLocation) -> Self {
        Self {
            has_changed: true,
            area_location,
            voxels: vec![Voxel::None; VOXELS_IN_AREA].into_boxed_slice(),
            max_height: vec![(AREA_HEIGHT - 1) as u8; (AREA_SIZE * AREA_SIZE) as usize]
                .into_boxed_slice(),
        }
    }

    pub fn sample_height(&self, local_x: u32, local_y: u32) -> u8 {
        self.max_height[(local_x + AREA_SIZE * local_y) as usize]
    }

    /// updates the max height for all columns
    pub fn update_all_column_heights(&mut self) {
        for y in 0..AREA_SIZE {
            for x in 0..AREA_SIZE {
                self.set_column_height(InternalLocation::new(x, y, 0));
            }
        }
    }

    fn set_column_height(&mut self, local_location: InternalLocation) {
        self.max_height[(local_location.x + local_location.y * AREA_SIZE) as usize] =
            self.calculate_column_height(local_location);
    }

    fn calculate_column_height(&self, local_location: InternalLocation) -> u8 {
        (0..AREA_HEIGHT)
            .find(|z| {
                !Voxel::TRANSPARENT.contains(&self.get(InternalLocation {
                    z: *z,
                    ..local_location
                }))
            })
            .unwrap_or(AREA_HEIGHT - 1) as u8
    }

    fn convert_to_index(local_location: InternalLocation) -> usize {
        (local_location.x + local_location.y * AREA_SIZE + local_location.z * AREA_SIZE * AREA_SIZE)
            as usize
    }

    pub fn get(&self, local_location: InternalLocation) -> Voxel {
        self.voxels[Self::convert_to_index(local_location)]
    }

    /// Used for batch modifications, doesn't update column heights.
    /// Should call `update_all_column_heights` at the end of the modifications.
    pub fn set_without_updating_max_height(
        &mut self,
        local_location: InternalLocation,
        voxel: Voxel,
    ) {
        self.voxels[Self::convert_to_index(local_location)] = voxel;
    }

    pub fn set(&mut self, local_location: InternalLocation, voxel: Voxel) {
        self.voxels[Self::convert_to_index(local_location)] = voxel;
        self.set_column_height(local_location);
    }

    pub fn get_area_location(&self) -> AreaLocation {
        self.area_location
    }

    pub fn get_x(&self) -> u32 {
        self.area_location.x
    }

    pub fn get_y(&self) -> u32 {
        self.area_location.y
    }

    /// Check if ALL neighbours WITHIN THE AREA are not transparent
    pub fn has_non_transparent_neighbours(&self, location: InternalLocation) -> bool {
        if location.x == 0
            || location.x + 1 >= AREA_SIZE
            || location.y == 0
            || location.y + 1 >= AREA_SIZE
            || location.z == 0
            || location.z + 1 >= AREA_HEIGHT
        {
            return false;
        }

        let current_voxel = self.get(location);
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x + 1,
                location.y,
                location.z,
            )),
        ) {
            return false;
        }
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x - 1,
                location.y,
                location.z,
            )),
        ) {
            return false;
        }
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x,
                location.y + 1,
                location.z,
            )),
        ) {
            return false;
        }
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x,
                location.y - 1,
                location.z,
            )),
        ) {
            return false;
        }
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x,
                location.y,
                location.z + 1,
            )),
        ) {
            return false;
        }
        if MeshGenerator::should_generate_face(
            current_voxel,
            self.get(InternalLocation::new(
                location.x,
                location.y,
                location.z - 1,
            )),
        ) {
            return false;
        }

        true
    }
}

#[derive(Debug, Encode, Decode)]
pub struct AreaDTO {
    pub voxels: Box<[Voxel]>,
}
impl AreaDTO {
    pub fn into_area(self, area_location: AreaLocation, has_changed: bool) -> Area {
        let mut area = Area {
            has_changed,
            area_location,
            voxels: self.voxels,
            max_height: vec![255; (AREA_SIZE * AREA_SIZE) as usize].into_boxed_slice(),
        };
        area.update_all_column_heights();

        area
    }
}
impl From<Area> for AreaDTO {
    fn from(value: Area) -> Self {
        Self {
            voxels: value.voxels,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_max_height() {
        let mut area = Area::new(AreaLocation::new(0, 0));
        area.set(InternalLocation::new(0, 0, 10), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 10);
        area.set(InternalLocation::new(0, 0, 5), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 5);
        area.set(InternalLocation::new(0, 0, 20), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 5);
        area.set(InternalLocation::new(1, 0, 1), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 5);
        assert_eq!(area.sample_height(1, 0), 1);
    }

    #[test]
    fn test_calculate_max_height_transparent() {
        let mut area = Area::new(AreaLocation::new(0, 0));
        area.set(InternalLocation::new(0, 0, 10), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 10);
        area.set(InternalLocation::new(0, 0, 5), Voxel::Glass);
        assert_eq!(area.sample_height(0, 0), 10);
        area.set(InternalLocation::new(0, 0, 8), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 8);
        area.set(InternalLocation::new(1, 0, 1), Voxel::Glass);
        assert_eq!(area.sample_height(1, 0), AREA_HEIGHT as u8 - 1);
    }

    #[test]
    fn test_set_without_calculating_max_height() {
        let mut area = Area::new(AreaLocation::new(0, 0));
        area.set_without_updating_max_height(InternalLocation::new(0, 0, 10), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), AREA_HEIGHT as u8 - 1);
        area.set_without_updating_max_height(InternalLocation::new(0, 0, 5), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), AREA_HEIGHT as u8 - 1);
        area.set(InternalLocation::new(0, 0, 20), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 5);
        area.set(InternalLocation::new(1, 0, 1), Voxel::Brick);
        assert_eq!(area.sample_height(0, 0), 5);
        assert_eq!(area.sample_height(1, 0), 1);
    }
}
