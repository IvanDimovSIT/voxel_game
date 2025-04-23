use macroquad::{
    color::Color,
    shapes::{draw_rectangle, draw_rectangle_lines},
};

use super::style::{BORDER_COLOR, SHADOW_COLOR, SHADOW_OFFSET};

pub fn is_point_in_rect(x: f32, y: f32, w: f32, h: f32, point_x: f32, point_y: f32) -> bool {
    (x..(x + w)).contains(&point_x) && (y..(y + h)).contains(&point_y)
}

pub fn get_text_width(text: &str, font_size: impl Into<f32>) -> f32 {
    text.len() as f32 * font_size.into() * 0.45
}

pub fn draw_rect_with_shadow(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x - SHADOW_OFFSET, y + SHADOW_OFFSET, w, h, SHADOW_COLOR);
    draw_rectangle(x, y, w, h, color);
    draw_rectangle_lines(x, y, w, h, 2.0, BORDER_COLOR);
}
