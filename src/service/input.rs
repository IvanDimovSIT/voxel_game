use macroquad::input::{
    is_key_down, is_key_pressed, is_key_released, is_mouse_button_down, is_mouse_button_pressed,
    mouse_wheel,
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

pub fn swim() -> bool {
    is_key_down(macroquad::input::KeyCode::Space)
}

pub fn exit_focus() -> bool {
    is_key_released(macroquad::input::KeyCode::Escape)
}

pub fn toggle_debug() -> bool {
    is_key_released(macroquad::input::KeyCode::GraveAccent)
}

pub fn is_start_place_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_pressed(macroquad::input::MouseButton::Right)
}

pub fn is_place_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_down(macroquad::input::MouseButton::Right)
}

pub fn is_start_replace_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_pressed(macroquad::input::MouseButton::Middle)
}

pub fn is_replace_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_down(macroquad::input::MouseButton::Middle)
}

pub fn is_start_destroy_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_pressed(macroquad::input::MouseButton::Left)
}

pub fn is_destroy_voxel(camera_controller: &CameraController) -> bool {
    camera_controller.is_focused() && is_mouse_button_down(macroquad::input::MouseButton::Left)
}

pub fn increase_render_distance() -> bool {
    is_key_released(macroquad::input::KeyCode::F2)
}

pub fn decrease_render_distance() -> bool {
    is_key_released(macroquad::input::KeyCode::F1)
}

pub fn is_enter_inventory() -> bool {
    is_key_released(macroquad::input::KeyCode::E)
}

pub fn is_enter_crafting() -> bool {
    is_key_released(macroquad::input::KeyCode::C)
}

pub fn get_number_key() -> Option<u8> {
    if is_key_pressed(macroquad::input::KeyCode::Key0) {
        return Some(0);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key1) {
        return Some(1);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key2) {
        return Some(2);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key3) {
        return Some(3);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key4) {
        return Some(4);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key5) {
        return Some(5);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key6) {
        return Some(6);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key7) {
        return Some(7);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key8) {
        return Some(8);
    }

    if is_key_pressed(macroquad::input::KeyCode::Key9) {
        return Some(9);
    }

    None
}

pub fn is_show_map() -> bool {
    is_key_down(macroquad::input::KeyCode::M)
}

#[derive(Debug, Clone, Copy)]
pub enum ScrollDirection {
    Up,
    Down,
    None,
}
pub fn get_scroll_direction() -> ScrollDirection {
    let (_x, y) = mouse_wheel();

    if y > 0.5 {
        ScrollDirection::Up
    } else if y < -0.5 {
        ScrollDirection::Down
    } else {
        ScrollDirection::None
    }
}
