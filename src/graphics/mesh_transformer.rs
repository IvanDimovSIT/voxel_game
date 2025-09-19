use macroquad::{math::Vec3, models::Mesh};

/// rotates a mesh and it's direction around z
pub fn rotate_around_z_with_direction(
    mesh: &mut Mesh,
    direction: &mut Vec3,
    origin: Vec3,
    angle: f32,
) {
    if angle <= f32::EPSILON {
        return;
    }

    let (sin_a, cos_a) = angle.sin_cos();
    rotate_mesh(mesh, origin, sin_a, cos_a);
    rotate_direction(direction, sin_a, cos_a);
}

/// rotates a mesh around z
pub fn rotate_around_z(mesh: &mut Mesh, origin: Vec3, angle: f32) {
    if angle <= f32::EPSILON {
        return;
    }

    let (sin_a, cos_a) = angle.sin_cos();
    rotate_mesh(mesh, origin, sin_a, cos_a);
}

fn rotate_mesh(mesh: &mut Mesh, origin: Vec3, sin: f32, cos: f32) {
    for v in &mut mesh.vertices {
        let p = &mut v.position;

        let dx = p.x - origin.x;
        let dy = p.y - origin.y;

        let new_x = dx * cos - dy * sin;
        let new_y = dx * sin + dy * cos;

        p.x = origin.x + new_x;
        p.y = origin.y + new_y;
    }
}

fn rotate_direction(direction: &mut Vec3, sin: f32, cos: f32) {
    debug_assert!(direction.is_normalized());
    let dir_x = direction.x;
    let dir_y = direction.y;

    direction.x = dir_x * cos - dir_y * sin;
    direction.y = dir_x * sin + dir_y * cos;
    *direction = direction.normalize_or_zero();
}

pub fn move_mesh(mesh: &mut Mesh, by: Vec3) {
    for v in &mut mesh.vertices {
        v.position += by;
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{PI, TAU};

    use macroquad::math::vec3;

    use crate::graphics::mesh_generator::MeshGenerator;

    use super::*;

    const LIMIT: f32 = 0.0001;

    #[test]
    fn test_move_mesh() {
        let mut mesh = MeshGenerator::generate_quad_mesh(1.0);
        let mesh_copy = MeshGenerator::generate_quad_mesh(1.0);

        move_mesh(&mut mesh, vec3(1.0, 2.0, 3.0));

        for (new, original) in mesh.vertices.iter().zip(mesh_copy.vertices.iter()) {
            assert_eq!(new.normal, original.normal);
            assert_eq!(new.color, original.color);
            assert_eq!(new.uv, original.uv);
            assert_eq!(new.position.x, original.position.x + 1.0);
            assert_eq!(new.position.y, original.position.y + 2.0);
            assert_eq!(new.position.z, original.position.z + 3.0);
        }

        assert_eq!(mesh.indices, mesh_copy.indices);
    }

    #[test]
    fn test_move_mesh_zero() {
        let mut mesh = MeshGenerator::generate_quad_mesh(1.0);
        let mesh_copy = MeshGenerator::generate_quad_mesh(1.0);

        move_mesh(&mut mesh, Vec3::ZERO);

        assert_mesh_eq(&mesh, &mesh_copy);
    }

    #[test]
    fn test_rotate_around_z_with_direction_no_rotation() {
        let mut mesh = MeshGenerator::generate_quad_mesh(1.0);
        let mut direction = vec3(1.0, 1.0, 1.0).normalize_or_zero();
        let mesh_copy = MeshGenerator::generate_quad_mesh(1.0);
        let direction_copy = direction.clone();

        rotate_around_z_with_direction(&mut mesh, &mut direction, vec3(0.0, 0.0, 0.0), 0.0);

        assert_mesh_eq(&mesh, &mesh_copy);
        assert_vec_equals(direction, direction_copy);
    }

    #[test]
    fn test_rotate_around_z_with_direction_full_rotation() {
        let mut mesh = MeshGenerator::generate_quad_mesh(1.0);
        let mut direction = vec3(1.0, 1.0, 1.0).normalize_or_zero();
        let mesh_copy = MeshGenerator::generate_quad_mesh(1.0);
        let direction_copy = direction.clone();

        rotate_around_z_with_direction(&mut mesh, &mut direction, vec3(0.0, 0.0, 0.0), TAU);

        assert_mesh_eq(&mesh, &mesh_copy);
        assert_vec_equals(direction, direction_copy);
    }

    #[test]
    fn test_rotate_around_z_with_direction_pi() {
        let mut mesh = MeshGenerator::generate_quad_mesh(1.0);
        let mut direction = vec3(1.0, 1.0, 1.0).normalize_or_zero();
        let mesh_copy = MeshGenerator::generate_quad_mesh(1.0);
        let direction_copy = direction.clone();

        rotate_around_z_with_direction(&mut mesh, &mut direction, vec3(0.0, 0.0, 0.0), PI);

        for (v1, v2) in mesh.vertices.iter().zip(mesh_copy.vertices.iter()) {
            assert_eq!(v1.normal, v2.normal);
            assert_eq!(v1.color, v2.color);
            assert_eq!(v1.uv, v2.uv);
            assert!((v1.position.x + v2.position.x) < LIMIT);
            assert!((v1.position.y + v2.position.y) < LIMIT);
            assert!((v1.position.z - v2.position.z).abs() < LIMIT);
        }

        assert_eq!(mesh.indices, mesh_copy.indices);

        assert!((direction.x + direction_copy.x) < LIMIT);
        assert!((direction.y + direction_copy.y) < LIMIT);
        assert!((direction.z - direction_copy.z).abs() < LIMIT);
    }

    fn assert_mesh_eq(mesh1: &Mesh, mesh2: &Mesh) {
        for (v1, v2) in mesh1.vertices.iter().zip(mesh2.vertices.iter()) {
            assert_eq!(v1.normal, v2.normal);
            assert_eq!(v1.color, v2.color);
            assert_eq!(v1.uv, v2.uv);
            assert_vec_equals(v1.position, v2.position);
        }

        assert_eq!(mesh1.indices, mesh2.indices);
    }

    fn assert_vec_equals(v1: Vec3, v2: Vec3) {
        assert!((v1.x - v2.x).abs() < LIMIT);
        assert!((v1.y - v2.y).abs() < LIMIT);
        assert!((v1.z - v2.z).abs() < LIMIT);
    }
}
