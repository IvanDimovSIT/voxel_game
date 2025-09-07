use macroquad::math::Vec3;

use crate::{
    model::{area::AREA_HEIGHT, location::Location, world::World},
    utils::vector_to_location,
};

const VOXEL_SIZE: f32 = 1.0;
const HALF_VOXEL_SIZE: f32 = VOXEL_SIZE / 2.0;

#[derive(Debug, Clone, Copy)]
pub enum RaycastResult {
    NoneHit,
    Hit {
        first_non_empty: Location,
        last_empty: Location,
    },
}

fn is_hit(world: &mut World, position: Location) -> bool {
    world.get(position).is_solid()
}

/// DDA raycasting
pub fn cast_ray(world: &mut World, from: Vec3, to: Vec3, max_distance: f32) -> RaycastResult {
    let ray = (to - from).normalize_or_zero();
    if ray == Vec3::ZERO {
        return RaycastResult::NoneHit;
    }

    let step_x = if ray.x < 0.0 { -1 } else { 1 };
    let step_y = if ray.y < 0.0 { -1 } else { 1 };
    let step_z = if ray.z < 0.0 { -1 } else { 1 };

    let mut current_position = vector_to_location(from);
    let mut previous_position = current_position;

    let next_boundary_x = if step_x < 0 {
        current_position.x as f32 - HALF_VOXEL_SIZE
    } else {
        current_position.x as f32 + HALF_VOXEL_SIZE
    };

    let next_boundary_y = if step_y < 0 {
        current_position.y as f32 - HALF_VOXEL_SIZE
    } else {
        current_position.y as f32 + HALF_VOXEL_SIZE
    };

    let next_boundary_z = if step_z < 0 {
        current_position.z as f32 - HALF_VOXEL_SIZE
    } else {
        current_position.z as f32 + HALF_VOXEL_SIZE
    };

    let mut t_max_x = if ray.x.abs() <= f32::EPSILON {
        f32::INFINITY
    } else {
        ((next_boundary_x - from.x) / ray.x).abs()
    };

    let mut t_max_y = if ray.y.abs() <= f32::EPSILON {
        f32::INFINITY
    } else {
        ((next_boundary_y - from.y) / ray.y).abs()
    };

    let mut t_max_z = if ray.z.abs() <= f32::EPSILON {
        f32::INFINITY
    } else {
        ((next_boundary_z - from.z) / ray.z).abs()
    };

    let delta_x = (VOXEL_SIZE / ray.x).abs();
    let delta_y = (VOXEL_SIZE / ray.y).abs();
    let delta_z = (VOXEL_SIZE / ray.z).abs();

    let mut distance_traveled = 0.0f32;

    if current_position.z < 0 || current_position.z >= AREA_HEIGHT as i32 {
        return RaycastResult::NoneHit;
    }
    if is_hit(world, current_position) {
        return RaycastResult::Hit {
            first_non_empty: current_position,
            last_empty: previous_position,
        };
    }

    while distance_traveled < max_distance {
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                previous_position = current_position;
                current_position.x += step_x;
                distance_traveled = t_max_x;
                t_max_x += delta_x;
            } else {
                previous_position = current_position;
                current_position.z += step_z;
                distance_traveled = t_max_z;
                t_max_z += delta_z;
            }
        } else if t_max_y < t_max_z {
            previous_position = current_position;
            current_position.y += step_y;
            distance_traveled = t_max_y;
            t_max_y += delta_y;
        } else {
            previous_position = current_position;
            current_position.z += step_z;
            distance_traveled = t_max_z;
            t_max_z += delta_z;
        }

        if current_position.z < 0 || current_position.z >= AREA_HEIGHT as i32 {
            return RaycastResult::NoneHit;
        }

        if is_hit(world, current_position) {
            return RaycastResult::Hit {
                first_non_empty: current_position,
                last_empty: previous_position,
            };
        }
    }

    RaycastResult::NoneHit
}
