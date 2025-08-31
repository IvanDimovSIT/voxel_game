use std::fmt::Write;

use macroquad::{
    color::Color,
    shapes::{draw_rectangle, draw_rectangle_lines},
    text::draw_text,
};

use crate::{
    interface::style::{CLEAR_SCREEN_COLOR, TEXT_COLOR},
    utils::use_str_buffer,
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

pub fn darken_background(width: f32, height: f32) {
    draw_rectangle(0.0, 0.0, width, height, CLEAR_SCREEN_COLOR);
}

pub fn draw_centered_multiline_text(
    text: &[&str],
    y: f32,
    width: f32,
    font_size: f32,
    color: Color,
) {
    for (i, line) in text.iter().enumerate() {
        let text_size = get_text_width(line, font_size);
        let line_x = (width - text_size) / 2.0;
        let line_y = y + i as f32 * font_size;
        draw_text(line, line_x, line_y, font_size, color);
    }
}

pub fn draw_item_name(x: f32, y: f32, voxel_name: &str, count: u8, font_size: f32) {
    const TEXT_BOX_X_OFFSET: f32 = 3.0;
    const TEXT_BOX_Y_OFFSET: f32 = -5.0;
    use_str_buffer(|buffer| {
        write!(buffer, "{voxel_name} ({count})").expect("error writing to text buffer");
        draw_rectangle(
            x,
            y - font_size,
            get_text_width(buffer, font_size) + TEXT_BOX_X_OFFSET,
            font_size,
            SHADOW_COLOR,
        );
        draw_text(
            buffer,
            x + TEXT_BOX_X_OFFSET,
            y + TEXT_BOX_Y_OFFSET,
            font_size,
            TEXT_COLOR,
        );
    });
}

pub fn draw_version_number(height: f32) {
    const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");
    const VERSION_NUMBER_TEXT_SIZE: f32 = 0.04;
    const OFFSET: f32 = 4.0;
    let font_size = VERSION_NUMBER_TEXT_SIZE * height;
    let y = height - OFFSET;

    draw_text(VERSION_NUMBER, OFFSET, y, font_size, TEXT_COLOR);
}
