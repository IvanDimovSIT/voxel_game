use bincode::{Decode, Encode};
use macroquad::math::{Vec3, vec3};

use crate::{model::world::World, utils::vector_to_location};

pub const LOCATION_OFFSET: i32 = 1_000_000;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Location {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}
impl From<InternalLocation> for Location {
    fn from(value: InternalLocation) -> Self {
        Self {
            x: value.x as i32 - LOCATION_OFFSET,
            y: value.y as i32 - LOCATION_OFFSET,
            z: value.z as i32,
        }
    }
}

impl From<Location> for Vec3 {
    fn from(val: Location) -> Self {
        vec3(val.x as f32, val.y as f32, val.z as f32)
    }
}

impl From<Vec3> for Location {
    fn from(value: Vec3) -> Self {
        vector_to_location(value)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode)]
pub struct InternalLocation {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}
impl InternalLocation {
    pub const fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub const fn offset_x(self, x: i32) -> Self {
        Self {
            x: (self.x as i32 + x) as _,
            y: self.y,
            z: self.z,
        }
    }

    pub const fn offset_y(self, y: i32) -> Self {
        Self {
            x: self.x,
            y: (self.y as i32 + y) as _,
            z: self.z,
        }
    }

    pub fn offset_z(self, z: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: (self.z as i32 + z) as _,
        }
    }
}
impl From<Location> for InternalLocation {
    fn from(value: Location) -> Self {
        Self {
            x: (value.x + LOCATION_OFFSET) as u32,
            y: (value.y + LOCATION_OFFSET) as u32,
            z: value.z as u32,
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AreaLocation {
    pub x: u32,
    pub y: u32,
}
impl AreaLocation {
    pub const fn new(x: u32, y: u32) -> Self {
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
