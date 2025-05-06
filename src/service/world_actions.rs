use crate::{
    graphics::renderer::Renderer,
    model::{area::AREA_HEIGHT, location::Location, voxel::Voxel, world::World},
};

use super::voxel_physics::VoxelSimulator;

pub fn place_voxel(
    location: Location,
    voxel: Voxel,
    camera_location: Location,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &VoxelSimulator,
) -> bool {
    let unable_to_place_voxel = world.get(location.into()) != Voxel::None
        || location == camera_location
        || location == Location::new(camera_location.x, camera_location.y, camera_location.z + 1)
        || voxel_simulator.location_has_voxel(location);

    if unable_to_place_voxel {
        return false;
    }

    world.set(location.into(), voxel);
    renderer.update_location(world, location.into());

    true
}

pub fn destroy_voxel(location: Location, world: &mut World, renderer: &mut Renderer) -> bool {
    if world.get(location.into()) == Voxel::None || location.z == AREA_HEIGHT as i32 - 1 {
        return false;
    }

    world.set(location.into(), Voxel::None);
    renderer.update_location(world, location.into());

    true
}
