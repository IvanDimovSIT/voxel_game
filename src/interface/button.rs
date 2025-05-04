use macroquad::{
    color::BLACK,
    input::{is_mouse_button_released, mouse_position},
    text::{TextParams, draw_text_ex},
};

use crate::service::sound_manager::{SoundId, SoundManager};

use super::{
    style::*,
    util::{draw_rect_with_shadow, is_point_in_rect},
};

/// draws a button and returns if it is pressed
pub fn draw_button(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    text: &str,
    text_size: u16,
    sound_manager: &SoundManager,
) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let is_hovered = is_point_in_rect(x, y, w, h, mouse_x, mouse_y);
    let button_color = if is_hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
    };

    draw_rect_with_shadow(x, y, w, h, button_color);
    draw_text_ex(
        text,
        x + MARGIN,
        y + h * 0.5 + text_size as f32 * 0.5,
        TextParams {
            font_size: text_size,
            color: BLACK,
            ..Default::default()
        },
    );

    let is_clicked = is_hovered && is_mouse_button_released(macroquad::input::MouseButton::Left);
    if is_clicked {
        sound_manager.play_sound(SoundId::Click);
    }

    is_clicked
}
