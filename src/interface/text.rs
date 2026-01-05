use macroquad::{
    color::Color,
    math::Vec2,
    text::{Font, TextParams, draw_text_ex, measure_text},
};

use crate::interface::style::{SHADOW_COLOR, TEXT_COLOR};

/// draws the version number in the left corner
pub fn draw_version_number(height: f32, font: &Font) {
    const VERSION_NUMBER: &str = env!("CARGO_PKG_VERSION");
    const VERSION_NUMBER_TEXT_SIZE: f32 = 0.04;
    const OFFSET: f32 = 4.0;
    let font_size = VERSION_NUMBER_TEXT_SIZE * height;
    let y = height - OFFSET;
    let version_text = format!("v{VERSION_NUMBER}");

    draw_game_text(&version_text, OFFSET, y, font_size, TEXT_COLOR, font);
}

/// draws text with the selected font
pub fn draw_game_text(
    text: &str,
    x: f32,
    y: f32,
    font_size: impl Into<f32>,
    color: Color,
    font: &Font,
) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: font_size.into() as u16,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_centered_multiline_text(
    text: &[&str],
    y: f32,
    width: f32,
    font_size: f32,
    color: Color,
    font: &Font,
) {
    for (i, line) in text.iter().enumerate() {
        let text_size = get_text_width(line, font_size, font);
        let line_x = (width - text_size) / 2.0;
        let line_y = y + i as f32 * font_size;
        draw_game_text(line, line_x, line_y, font_size, color, font);
    }
}

pub fn get_text_width(text: &str, font_size: impl Into<f32>, font: &Font) -> f32 {
    measure_text(text, Some(font), font_size.into() as u16, 1.0).width
}

pub fn draw_text_with_shadow(
    text: &str,
    pos: Vec2,
    offset: Vec2,
    font_size: impl Into<f32>,
    color: Color,
    font: &Font,
) {
    let font_size = font_size.into();
    let shadow_position = pos + offset;
    draw_game_text(
        text,
        shadow_position.x,
        shadow_position.y,
        font_size,
        SHADOW_COLOR,
        font,
    );
    draw_game_text(text, pos.x, pos.y, font_size, color, font);
}

pub fn draw_multiline_left_text(
    text: &[&str],
    y: f32,
    width: f32,
    font_size: f32,
    color: Color,
    font: &Font,
) {
    let max_width = text
        .iter()
        .map(|s| get_text_width(s, font_size, font) as i32)
        .max()
        .unwrap_or(0) as f32;

    let x = (width - max_width) / 2.0;
    for (index, s) in text.iter().enumerate() {
        let y_text = font_size * index as f32 + y;
        draw_game_text(s, x, y_text, font_size, color, font);
    }
}
