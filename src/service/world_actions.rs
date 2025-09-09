use macroquad::math::vec3;

use crate::{
    graphics::{renderer::Renderer, voxel_particle_system::VoxelParticleSystem},
    model::{
        area::AREA_HEIGHT, location::Location, player_info::PlayerInfo, voxel::Voxel, world::World,
    },
    service::physics::{
        player_physics::will_new_voxel_cause_collision, voxel_simulator::VoxelSimulator,
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
    let unable_to_place_voxel = world.get(location).is_solid()
        || will_new_voxel_cause_collision(player_info, location)
        || voxel_simulator.location_has_voxel(location);

    if unable_to_place_voxel {
        return false;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    true
}

pub fn replace_voxel(
    location: Location,
    voxel: Voxel,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
) -> Option<Voxel> {
    debug_assert!(voxel != Voxel::None);
    let to_be_replaced = world.get(location);
    if !to_be_replaced.is_solid() || location.z == AREA_HEIGHT as i32 - 1 || to_be_replaced == voxel
    {
        return None;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    Some(to_be_replaced)
}

pub fn destroy_voxel(
    location: Location,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
    voxel_particles: &mut VoxelParticleSystem,
) -> Option<Voxel> {
    let voxel = world.get(location);
    if voxel == Voxel::None || location.z == AREA_HEIGHT as i32 - 1 {
        return None;
    }

    world.set(location, Voxel::None);
    voxel_particles.add_particles_for_destroyed(voxel, location, renderer.get_mesh_generator());
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    Some(voxel)
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

            if Voxel::WATER.contains(&voxel) {
                world.set(bottom_location, Voxel::Stone);
            }
            return;
        } else {
            player_info
                .camera_controller
                .set_position(bottom_position + vec3(0.0, 0.0, 1.0));
        }
    }
}

pub fn update_player_in_water(player_info: &mut PlayerInfo, world: &mut World) {
    let player_location = player_info.camera_controller.get_camera_voxel_location();
    player_info.is_in_water = Voxel::WATER.contains(&world.get(player_location));
}
