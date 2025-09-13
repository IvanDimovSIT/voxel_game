use macroquad::{
    math::{Vec3, vec3},
    prelude::error,
};

use crate::{
    model::{location::Location, player_info::PlayerInfo, voxel::Voxel, world::World},
    utils::{StackVec, vector_to_location},
};

const HALF_VOXEL_SIZE: f32 = 0.5;
pub const GRAVITY: f32 = 25.0;
pub const MAX_FALL_SPEED: f32 = 60.0;
const STRONG_COLLISION_SPEED: f32 = MAX_FALL_SPEED * 0.2;
const MAX_LOCATIONS_TO_CHECK: usize = 9;
const MOVE_CHECKS: usize = 4;
const MIN_VELOCITY_TO_BOUNCE: f32 = 1.5;
const MAX_SWIM_SPEED: f32 = -25.0;
const GAIN_SWIM_SPEED: f32 = -25.0;
const IN_WATER_FALL_SPEED_MODIFIER: f32 = 0.2;
const IN_WATER_MAX_FALL_SPEED: f32 = 15.0;
const IN_WATER_MOVE_SPEED_MODIFIER: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionType {
    None,
    Weak,
    Strong,
    Bounce,
}

pub fn push_player_up_if_stuck(player_info: &mut PlayerInfo, world: &mut World) {
    let down_position = player_info.camera_controller.get_position() + vec3(0.0, 0.0, 1.0);
    let down_location: Location = vector_to_location(down_position);
    if !is_location_non_empty(down_location, world) {
        return;
    }

    error!("Player is stuck!");
    player_info
        .camera_controller
        .set_position(down_position - vec3(0.0, 0.0, 2.5));
    push_player_up_if_stuck(player_info, world);
}

/// process vertical collisions for the player
pub fn process_collisions(
    player_info: &mut PlayerInfo,
    world: &mut World,
    delta: f32,
) -> CollisionType {
    player_info.velocity = calculate_fall_velocity(player_info, delta);

    let top_position =
        player_info.camera_controller.get_position() + vec3(0.0, 0.0, player_info.velocity * delta);
    let down_position = top_position + vec3(0.0, 0.0, 1.5);

    let mut down_locations = StackVec::new();
    let mut top_locations = StackVec::new();
    find_locations_for_collisions(down_position, player_info.size, &mut down_locations);
    find_locations_for_collisions(top_position, player_info.size, &mut top_locations);

    for down_location in down_locations {
        let voxel_hit = world.get(down_location);
        if voxel_hit.is_solid() {
            if voxel_hit == Voxel::Trampoline && should_bounce_from_trampoline(player_info) {
                return CollisionType::Bounce;
            }

            let mut collision_type = CollisionType::Weak;
            if player_info.velocity >= STRONG_COLLISION_SPEED {
                collision_type = CollisionType::Strong;
            }
            player_info.velocity = 0.0;
            player_info.camera_controller.set_position(
                vec3(top_position.x, top_position.y, down_location.z as f32) - vec3(0.0, 0.0, 2.0),
            );
            return collision_type;
        }
    }

    for top_location in top_locations {
        if is_location_non_empty(top_location, world) {
            player_info.velocity = 0.0;
            return CollisionType::Weak;
        }
    }

    player_info.camera_controller.set_position(top_position);
    CollisionType::None
}

fn calculate_fall_velocity(player_info: &PlayerInfo, delta: f32) -> f32 {
    let (water_modifier, max_fall_speed) = if player_info.is_in_water {
        (IN_WATER_FALL_SPEED_MODIFIER, IN_WATER_MAX_FALL_SPEED)
    } else {
        (1.0, MAX_FALL_SPEED)
    };

    (player_info.velocity + GRAVITY * delta * water_modifier).min(max_fall_speed)
}

fn should_bounce_from_trampoline(player_info: &mut PlayerInfo) -> bool {
    if player_info.velocity < MIN_VELOCITY_TO_BOUNCE {
        return false;
    }

    player_info.velocity = -player_info.velocity;
    true
}

pub fn try_jump(player_info: &mut PlayerInfo, world: &mut World) {
    if player_info.is_in_water {
        return;
    }

    let bottom_voxel_position = player_info.camera_controller.get_bottom_position();
    let mut down_locations = StackVec::new();
    find_locations_for_collisions(bottom_voxel_position, player_info.size, &mut down_locations);

    let is_on_ground = down_locations
        .into_iter()
        .any(|location| is_location_non_empty(location, world));

    if is_on_ground {
        player_info.velocity = player_info.jump_velocity;
    }
}

pub fn try_swim(player_info: &mut PlayerInfo, delta: f32) {
    if !player_info.is_in_water {
        return;
    }

    player_info.velocity += delta * GAIN_SWIM_SPEED;
    player_info.velocity = player_info.velocity.max(MAX_SWIM_SPEED);
}

/// checks if the new voxel location will cause a collision with the player
pub fn will_new_voxel_cause_collision(
    player_info: &PlayerInfo,
    new_voxel_location: Location,
) -> bool {
    let top_position = player_info.camera_controller.get_position();
    let down_position = top_position + vec3(0.0, 0.0, 1.0);
    let mut down_locations = StackVec::new();
    let mut top_locations = StackVec::new();
    find_locations_for_collisions(down_position, player_info.size, &mut down_locations);
    find_locations_for_collisions(top_position, player_info.size, &mut top_locations);

    down_locations
        .into_iter()
        .chain(top_locations)
        .any(|location| location == new_voxel_location)
}

/// move and process horizontal collisions for the player
pub fn try_move(player_info: &mut PlayerInfo, world: &mut World, displacement: Vec3) {
    let top_position = player_info.camera_controller.get_position();
    let bottom_position =
        player_info.camera_controller.get_bottom_position() - vec3(0.0, 0.0, 0.55);
    let modified_displacement = modify_displacement_in_water(displacement, player_info);
    let mut top_displaced = top_position + modified_displacement;
    let mut bottom_displaced = bottom_position + modified_displacement;
    let mut top_locations;
    let mut bottom_locations;

    let delta_displacement =
        modified_displacement * (modified_displacement.length() / MOVE_CHECKS as f32);
    for _checks in 0..MOVE_CHECKS {
        top_locations = StackVec::new();
        bottom_locations = StackVec::new();
        find_locations_for_collisions(top_displaced, player_info.size, &mut top_locations);
        find_locations_for_collisions(bottom_displaced, player_info.size, &mut bottom_locations);

        let any_collision = top_locations
            .into_iter()
            .chain(bottom_locations)
            .any(|location| is_location_non_empty(location, world));

        if any_collision {
            top_displaced -= delta_displacement;
            bottom_displaced -= delta_displacement;
        } else {
            player_info.camera_controller.set_position(top_displaced);
            return;
        }
    }
}

fn modify_displacement_in_water(displacement: Vec3, player_info: &PlayerInfo) -> Vec3 {
    if !player_info.is_in_water {
        return displacement;
    }

    displacement * IN_WATER_MOVE_SPEED_MODIFIER
}

fn is_location_non_empty(location: Location, world: &mut World) -> bool {
    world.get(location).is_solid()
}

/// finds locations around the position that could cause collisions
fn find_locations_for_collisions(
    position: Vec3,
    size: f32,
    locations: &mut StackVec<Location, MAX_LOCATIONS_TO_CHECK>,
) {
    debug_assert!(locations.is_empty());
    let x_round = position.x.round();
    let y_round = position.y.round();
    let x_min = x_round - HALF_VOXEL_SIZE;
    let x_max = x_round + HALF_VOXEL_SIZE;
    let y_min = y_round - HALF_VOXEL_SIZE;
    let y_max = y_round + HALF_VOXEL_SIZE;

    locations.push(vector_to_location(position));
    if position.x + size >= x_max {
        if position.y + size >= y_max {
            locations.push(vector_to_location(position + vec3(size, size, 0.0)));
        }
        if position.y - size <= y_min {
            locations.push(vector_to_location(position + vec3(size, -size, 0.0)));
        }
        locations.push(vector_to_location(position + vec3(size, 0.0, 0.0)));
    }

    if position.x - size <= x_min {
        if position.y + size >= y_max {
            locations.push(vector_to_location(position + vec3(-size, size, 0.0)));
        }
        if position.y - size <= y_min {
            locations.push(vector_to_location(position + vec3(-size, -size, 0.0)));
        }
        locations.push(vector_to_location(position + vec3(-size, 0.0, 0.0)));
    }

    if position.y + size >= y_max {
        locations.push(vector_to_location(position + vec3(0.0, size, 0.0)));
    }

    if position.y - size <= y_min {
        locations.push(vector_to_location(position + vec3(0.0, -size, 0.0)));
    }
}
