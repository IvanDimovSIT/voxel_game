use macroquad::{
    math::{Vec3, Vec3Swizzles, vec3},
    models::Mesh,
};

use crate::{
    graphics::mesh_manager::MeshManager,
    model::{location::Location, player_info::PlayerInfo, voxel::Voxel, world::World},
    service::creatures::creature_manager::CreatureDTO,
    utils::vector_to_location,
};

const PUSH_FROM_POINT_SPEED: f32 = 2.0;

pub trait Creature {
    fn update(&mut self, delta: f32, world: &mut World, player_info: &PlayerInfo);
    fn get_mesh_with_index(&self) -> (&Mesh, usize);
    fn get_position(&self) -> Vec3;
    fn get_size(&self) -> Vec3;
    fn get_allowed_spawn_voxels() -> &'static [Voxel]
    where
        Self: Sized;
    fn create_dto(&self) -> Option<CreatureDTO>;
    fn from_dto(creature_dto: CreatureDTO, mesh_manager: &MeshManager) -> Option<Box<dyn Creature>>
    where
        Self: Sized;
}

/// returns the position of collision
pub fn collides(creature: &impl Creature, world: &mut World) -> Option<Vec3> {
    let pos = creature.get_position();
    let size = creature.get_size();
    let creature_location: Location = pos.into();
    let half_x = size.x * 0.5;
    let half_y = size.y * 0.5;

    world.with_cached_area(creature_location, |world, cached_area| {
        let positions_to_check = [
            pos,
            pos + vec3(half_x, 0.0, 0.0),
            pos - vec3(half_x, 0.0, 0.0),
            pos + vec3(0.0, half_y, 0.0),
            pos - vec3(0.0, half_y, 0.0),
            pos + vec3(half_x, half_y, 0.0),
            pos + vec3(half_x, -half_y, 0.0),
            pos + vec3(-half_x, half_y, 0.0),
            pos + vec3(-half_x, -half_y, 0.0),
        ];

        positions_to_check.into_iter().find(|loc| {
            world
                .get_with_cache(Into::<Location>::into(*loc), Some(cached_area))
                .is_solid()
        })
    })
}

pub fn collides_with_player(creature: &impl Creature, player_info: &PlayerInfo) -> bool {
    let player_pos = player_info.camera_controller.get_bottom_position();
    let player_half_size = PlayerInfo::PLAYER_SIZE;
    let player_height = 1.0;

    let position = creature.get_position();
    let size = creature.get_size();
    let half_x = size.x * 0.5;
    let half_y = size.y * 0.5;
    let half_z = size.z * 0.5;

    let min_x = position.x - half_x;
    let max_x = position.x + half_x;
    let min_y = position.y - half_y;
    let max_y = position.y + half_y;
    let min_z = position.z - half_z;
    let max_z = position.z + half_z;

    let no_collision = player_pos.x - player_half_size > max_x
        || player_pos.x + player_half_size < min_x
        || player_pos.y - player_half_size > max_y
        || player_pos.y + player_half_size < min_y
        || player_pos.z - player_height > max_z
        || player_pos.z + player_height < min_z;

    !no_collision
}

/// returns the change in position
pub fn push_away_from(creature: &impl Creature, point: Vec3, delta: f32) -> Vec3 {
    let position = creature.get_position();
    let dir_2d = position.xy() - point.xy();
    let dir_to_creature = vec3(dir_2d.x, dir_2d.y, 0.0).normalize_or_zero();

    dir_to_creature * delta * PUSH_FROM_POINT_SPEED
}

/// checks for static wall collisions and returns displacement
pub fn perform_static_collisions(
    creature: &impl Creature,
    delta: f32,
    world: &mut World,
    start_position: Vec3,
) -> Vec3 {
    const NO_MOVEMENT_THRESHOLD: f32 = 0.01;
    let creature_position = creature.get_position();
    if (start_position.x - creature_position.x).abs() > NO_MOVEMENT_THRESHOLD
        || (start_position.y - creature_position.y).abs() > NO_MOVEMENT_THRESHOLD
    {
        return Vec3::ZERO;
    }

    let collision = collides(creature, world);
    if let Some(point) = collision {
        push_away_from(creature, point, delta)
    } else {
        Vec3::ZERO
    }
}

/// returns new creature z and if it's on the ground
pub fn collides_with_ground(creature: &impl Creature, world: &mut World) -> (f32, bool) {
    let position = creature.get_position();
    let size = creature.get_size();
    let half_z = size.z * 0.5;
    let below = vec3(position.x, position.y, position.z + half_z);
    let above = vec3(position.x, position.y, position.z - half_z);

    let bottom_location = vector_to_location(below);
    let top_location = vector_to_location(above);

    let bottom_voxel = world.get(bottom_location);

    if !bottom_voxel.is_solid() {
        let top_voxel = world.get(top_location);
        let result = if top_voxel.is_solid() {
            top_location.z as f32 + Voxel::HALF_SIZE + half_z
        } else {
            position.z
        };

        return (result, false);
    }

    (bottom_location.z as f32 - Voxel::HALF_SIZE - half_z, true)
}
