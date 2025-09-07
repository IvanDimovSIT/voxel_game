use macroquad::{
    color::Color,
    math::vec2,
    shapes::draw_rectangle,
    texture::{DrawTextureParams, draw_texture_ex},
};

use crate::{
    graphics::texture_manager::TextureManager, interface::style::CLEAR_SCREEN_COLOR,
    model::voxel::Voxel,
};

const WATER_SCREEN_EFFECT_COLOR: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 0.8,
};

pub fn darken_background(width: f32, height: f32) {
    draw_rectangle(0.0, 0.0, width, height, CLEAR_SCREEN_COLOR);
}

pub fn draw_water_effect(width: f32, height: f32, texture_manager: &TextureManager) {
    let water_texture = texture_manager.get(Voxel::WaterSource);

    draw_texture_ex(
        &water_texture,
        0.0,
        0.0,
        WATER_SCREEN_EFFECT_COLOR,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );
}
