use macroquad::input::{
    is_key_down, is_key_pressed, is_key_released, is_mouse_button_pressed, is_mouse_button_released, mouse_wheel,
};

use super::camera_controller::CameraController;

pub fn move_left() -> bool {
    is_key_down(macroquad::input::KeyCode::A) || is_key_down(macroquad::input::KeyCode::Left)
}

pub fn move_right() -> bool {
    is_key_down(macroquad::input::KeyCode::D) || is_key_down(macroquad::input::KeyCode::Right)
}

pub fn move_forward() -> bool {
    is_key_down(macroquad::input::KeyCode::W) || is_key_down(macroquad::input::KeyCode::Up)
}

pub fn move_back() -> bool {
    is_key_down(macroquad::input::KeyCode::S) || is_key_down(macroquad::input::KeyCode::Down)
}

pub fn jump() -> bool {
    is_key_pressed(macroquad::input::KeyCode::Space)
}

pub fn enter_focus() -> bool {
    is_mouse_button_released(macroquad::input::MouseButton::Left)
}

pub fn exit_focus() -> bool {
    is_key_down(macroquad::input::KeyCode::Escape)
}

pub fn toggle_debug() -> bool {
    is_key_released(macroquad::input::KeyCode::GraveAccent)
}

pub fn is_place_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_pressed(macroquad::input::MouseButton::Right)
}

pub fn is_destroy_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_pressed(macroquad::input::MouseButton::Left)
}

pub fn increase_render_distance() -> bool {
    is_key_released(macroquad::input::KeyCode::F2)
}

pub fn decrease_render_distance() -> bool {
    is_key_released(macroquad::input::KeyCode::F1)
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
    None
}
pub fn get_scroll_direction() -> ScrollDirection {
    let (_x, y) = mouse_wheel();

    println!("{y}");
    if y > 0.5 {
        ScrollDirection::Up
    } else if y < -0.5 {
        ScrollDirection::Down
    } else {
        ScrollDirection::None
    }
}