use bincode::{Decode, Encode};

use crate::graphics::mesh_generator::MeshGenerator;

use super::{
    location::{InternalLocation, Location},
    voxel::Voxel,
    world::World,
};

pub const AREA_SIZE: u32 = 16;
pub const AREA_HEIGHT: u32 = 64;
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

#[derive(Debug)]
pub struct Area {
    pub has_changed: bool,
    area_location: AreaLocation,
    voxels: Box<[Voxel]>,
}
impl Area {
    pub fn new(area_location: AreaLocation) -> Self {
        Self {
            has_changed: true,
            area_location,
            voxels: vec![Voxel::None; VOXELS_IN_AREA].into_boxed_slice(),
        }
    }

    fn convert_to_index(local_location: InternalLocation) -> usize {
        (local_location.x + local_location.y * AREA_SIZE + local_location.z * AREA_SIZE * AREA_SIZE)
            as usize
    }

    pub fn get(&self, local_location: InternalLocation) -> Voxel {
        self.voxels[Self::convert_to_index(local_location)]
    }

    pub fn set(&mut self, local_location: InternalLocation, voxel: Voxel) {
        self.voxels[Self::convert_to_index(local_location)] = voxel;
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
        Area {
            has_changed,
            area_location,
            voxels: self.voxels,
        }
    }
}
impl From<Area> for AreaDTO {
    fn from(value: Area) -> Self {
        Self {
            voxels: value.voxels,
        }
    }
}
