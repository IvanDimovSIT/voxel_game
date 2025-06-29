use macroquad::{math::vec3, prelude::error};

use crate::{
    graphics::renderer::Renderer,
    model::{
        area::AREA_HEIGHT, location::Location, player_info::PlayerInfo,
        user_settings::UserSettings, voxel::Voxel, world::World,
    },
    service::sound_manager::{SoundId, SoundManager},
    utils::vector_to_location,
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
    let unable_to_place_voxel = world.get(location) != Voxel::None
        || location == camera_location
        || location == Location::new(camera_location.x, camera_location.y, camera_location.z + 1)
        || voxel_simulator.location_has_voxel(location);

    if unable_to_place_voxel {
        return false;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);

    true
}

pub fn destroy_voxel(location: Location, world: &mut World, renderer: &mut Renderer) -> bool {
    if world.get(location) == Voxel::None || location.z == AREA_HEIGHT as i32 - 1 {
        return false;
    }

    world.set(location, Voxel::None);
    renderer.update_location(world, location);

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

pub fn process_collisions(
    player_info: &mut PlayerInfo,
    world: &mut World,
    sound_manager: &SoundManager,
    user_settings: &UserSettings,
    delta: f32,
) {
    const GRAVITY: f32 = 25.0;
    const MAX_FALL_SPEED: f32 = 60.0;

    player_info.velocity = (player_info.velocity + GRAVITY * delta).min(MAX_FALL_SPEED);

    let top_position =
        player_info.camera_controller.get_position() + vec3(0.0, 0.0, player_info.velocity * delta);
    let down_position = top_position + vec3(0.0, 0.0, 1.5);

    let down_location = vector_to_location(down_position);
    let down_voxel = world.get(down_location);
    if down_voxel != Voxel::None {
        if player_info.velocity > MAX_FALL_SPEED * 0.2 {
            sound_manager.play_sound(SoundId::Fall, user_settings);
        }
        player_info.velocity = 0.0;
        player_info.camera_controller.set_position(
            vec3(top_position.x, top_position.y, down_location.z as f32) - vec3(0.0, 0.0, 2.0),
        );
        return;
    }

    let top_location = vector_to_location(top_position);
    let top_voxel = world.get(top_location);
    if top_voxel != Voxel::None {
        player_info.velocity = 0.0;
        return;
    }

    player_info.camera_controller.set_position(top_position);
}

pub fn push_player_up_if_stuck(player_info: &mut PlayerInfo, world: &mut World) {
    let down_position = player_info.camera_controller.get_position() + vec3(0.0, 0.0, 1.0);
    let down_location: Location = vector_to_location(down_position);
    let voxel = world.get(down_location);
    if voxel == Voxel::None {
        return;
    }

    error!("Player is stuck!");
    player_info
        .camera_controller
        .set_position(down_position - vec3(0.0, 0.0, 2.5));
    push_player_up_if_stuck(player_info, world);
}
