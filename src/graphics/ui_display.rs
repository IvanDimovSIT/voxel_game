use macroquad::{
    camera::Camera3D, color::WHITE, math::vec3, models::draw_cube_wires, shapes::draw_circle,
};

use crate::model::location::Location;

pub fn draw_crosshair(width: f32, height: f32) {
    draw_circle(width / 2.0, height / 2.0, 2.0, WHITE);
}

pub fn draw_selected_voxel(location: Location, camera: &Camera3D) {
    let position = vec3(
        location.x as f32 - camera.position.x,
        location.y as f32 - camera.position.y,
        location.z as f32 - camera.position.z,
    );
    draw_cube_wires(position, vec3(1.0, 1.0, 1.0), WHITE);
}
