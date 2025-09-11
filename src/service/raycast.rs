use macroquad::math::Vec3;

use crate::{
    model::{area::AREA_HEIGHT, location::Location, voxel::Voxel, world::World},
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
    let voxel = world.get(position);

    voxel != Voxel::None && !Voxel::WATER.contains(&voxel)
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

#[cfg(test)]
mod tests {
    use macroquad::math::vec3;

    use crate::model::area::Area;

    use super::*;

    #[test]
    fn test_cast_ray() {
        let world_name = "test_world_test_cast_ray";
        let mut world = World::new(world_name);

        let voxel_location = Location::new(5, 5, 10);
        let mut area = Area::new(voxel_location.into());
        area.set(
            World::convert_global_to_local_location(voxel_location.into()),
            Voxel::Brick,
        );
        world.return_area(area);

        let result1 = cast_ray(&mut world, vec3(5.0, 5.0, 1.0), vec3(5.0, 5.0, 10.0), 9.0);
        match result1 {
            RaycastResult::Hit {
                first_non_empty,
                last_empty,
            } => {
                assert_eq!(first_non_empty, voxel_location);
                assert_eq!(last_empty, Location::new(5, 5, 9));
            }
            _ => panic!("should be hit"),
        }

        let result2 = cast_ray(&mut world, vec3(5.0, 5.0, 1.0), vec3(5.0, 5.0, 10.0), 5.0);
        assert!(matches!(result2, RaycastResult::NoneHit));

        let result3 = cast_ray(&mut world, vec3(5.0, 5.0, 1.0), vec3(10.0, 5.0, 10.0), 10.0);
        assert!(matches!(result3, RaycastResult::NoneHit));
    }

    #[test]
    fn test_cast_ray_out_of_height() {
        let world_name = "test_world_test_cast_ray_out_of_height";
        let mut world = World::new(world_name);

        let voxel_location = Location::new(5, 5, 10);
        let area = Area::new(voxel_location.into());
        world.return_area(area);

        let result1 = cast_ray(&mut world, vec3(5.0, 5.0, 1.0), vec3(5.0, 5.0, 0.0), 5.0);

        assert!(matches!(result1, RaycastResult::NoneHit));

        let result2 = cast_ray(
            &mut world,
            vec3(5.0, 5.0, AREA_HEIGHT as f32 - 1.0),
            vec3(5.0, 5.0, AREA_HEIGHT as f32),
            10.0,
        );

        assert!(matches!(result2, RaycastResult::NoneHit));
    }
}
