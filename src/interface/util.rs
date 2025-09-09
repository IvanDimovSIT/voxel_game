use std::fmt::Write;

use macroquad::{
    color::Color,
    shapes::{draw_rectangle, draw_rectangle_lines},
    text::Font,
};

use crate::{
    interface::{
        style::TEXT_COLOR,
        text::{draw_game_text, get_text_width},
    },
    utils::use_str_buffer,
};

use super::style::{BORDER_COLOR, SHADOW_COLOR, SHADOW_OFFSET};

pub fn is_point_in_rect(x: f32, y: f32, w: f32, h: f32, point_x: f32, point_y: f32) -> bool {
    (x..(x + w)).contains(&point_x) && (y..(y + h)).contains(&point_y)
}

pub fn draw_rect_with_shadow(x: f32, y: f32, w: f32, h: f32, color: Color) {
    draw_rectangle(x - SHADOW_OFFSET, y + SHADOW_OFFSET, w, h, SHADOW_COLOR);
    draw_rectangle(x, y, w, h, color);
    draw_rectangle_lines(x, y, w, h, 2.0, BORDER_COLOR);
}

pub fn draw_item_name_box(
    x: f32,
    y: f32,
    voxel_name: &str,
    count: u8,
    font_size: f32,
    font: &Font,
) {
    const TEXT_BOX_X_OFFSET: f32 = 3.0;
    const TEXT_BOX_Y_OFFSET: f32 = -5.0;
    use_str_buffer(|buffer| {
        write!(buffer, "{voxel_name} ({count})").expect("error writing to text buffer");
        draw_rectangle(
            x,
            y - font_size,
            get_text_width(buffer, font_size, font) + TEXT_BOX_X_OFFSET,
            font_size,
            SHADOW_COLOR,
        );
        draw_game_text(
            buffer,
            x + TEXT_BOX_X_OFFSET,
            y + TEXT_BOX_Y_OFFSET,
            font_size,
            TEXT_COLOR,
            font,
        );
    });
}
