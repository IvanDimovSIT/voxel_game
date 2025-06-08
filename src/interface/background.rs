use macroquad::{
    color::WHITE,
    math::vec2,
    texture::{DrawTextureParams, draw_texture_ex},
};

use crate::graphics::texture_manager::TextureManager;

/// draws the inteface background picture
pub fn draw_background(width: f32, height: f32, texture_manager: &TextureManager) {
    let background_texture = texture_manager.get_title_screen_background();
    draw_texture_ex(
        &background_texture,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );
}
