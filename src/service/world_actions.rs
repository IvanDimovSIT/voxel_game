use macroquad::math::vec3;

use crate::{
    graphics::renderer::Renderer,
    model::{
        area::AREA_HEIGHT, location::Location, player_info::PlayerInfo, voxel::Voxel, world::World,
    },
    service::physics::{
        player_physics::will_new_voxel_cause_collision, voxel_physics::VoxelSimulator,
    },
    utils::vector_to_location,
};

pub fn place_voxel(
    location: Location,
    voxel: Voxel,
    player_info: &PlayerInfo,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
) -> bool {
    debug_assert!(voxel != Voxel::None);
    let unable_to_place_voxel = world.get(location) != Voxel::None
        || will_new_voxel_cause_collision(player_info, location)
        || voxel_simulator.location_has_voxel(location);

    if unable_to_place_voxel {
        return false;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_voxels(world, renderer, location);

    true
}

pub fn replace_voxel(
    location: Location,
    voxel: Voxel,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
) -> bool {
    debug_assert!(voxel != Voxel::None);
    let to_be_replaced = world.get(location);
    if to_be_replaced == Voxel::None
        || location.z == AREA_HEIGHT as i32 - 1
        || to_be_replaced == voxel
    {
        return false;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_voxels(world, renderer, location);

    true
}

pub fn destroy_voxel(
    location: Location,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
) -> bool {
    if world.get(location) == Voxel::None || location.z == AREA_HEIGHT as i32 - 1 {
        return false;
    }

    world.set(location, Voxel::None);
    renderer.update_location(world, location);
    voxel_simulator.update_voxels(world, renderer, location);

    true
}

pub fn put_player_on_ground(player_info: &mut PlayerInfo, world: &mut World) {
    loop {
        let bottom_position = player_info.camera_controller.get_bottom_position();
        let bottom_location = vector_to_location(bottom_position);
        if bottom_location.z + 1 >= AREA_HEIGHT as i32 {
            return;
        }
        let voxel = world.get(bottom_location);

        if voxel != Voxel::None {
            player_info
                .camera_controller
                .set_position(bottom_position - vec3(0.0, 0.0, 1.0));
            return;
        } else {
            player_info
                .camera_controller
                .set_position(bottom_position + vec3(0.0, 0.0, 1.0));
        }
    }
}
