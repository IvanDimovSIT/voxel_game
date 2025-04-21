use macroquad::{
    color::BLACK,
    input::{is_mouse_button_released, mouse_position},
    shapes::{draw_rectangle, draw_rectangle_lines},
    text::{TextParams, draw_text_ex},
};

use super::{style::*, util::is_point_in_rect};

/// draws a button and returns if it is pressed
pub fn draw_button(x: f32, y: f32, w: f32, h: f32, text: &str, text_size: u16) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let is_hovered = is_point_in_rect(x, y, w, h, mouse_x, mouse_y);
    let button_color = if is_hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
    };

    draw_rectangle(x - SHADOW_OFFSET, y + SHADOW_OFFSET, w, h, SHADOW_COLOR);
    draw_rectangle(x, y, w, h, button_color);
    draw_rectangle_lines(x, y, w, h, 2.0, BORDER_COLOR);
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

    is_hovered && is_mouse_button_released(macroquad::input::MouseButton::Left)
}
