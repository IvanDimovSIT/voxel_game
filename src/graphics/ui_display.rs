use bincode::{Decode, Encode};
use macroquad::{
    camera::Camera3D,
    color::{Color, WHITE},
    math::{vec2, vec3},
    miniquad::window::screen_size,
    models::draw_cube_wires,
    shapes::{draw_circle, draw_rectangle},
    text::Font,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};
use std::fmt::Write;

use crate::{
    interface::{style::TEXT_COLOR, text::draw_game_text},
    model::{
        inventory::{Inventory, Item},
        location::Location,
    },
    service::asset_manager::AssetManager,
    utils::use_str_buffer,
};

const BASE_COUNT_FONT_SIZE: f32 = 0.5;

pub fn draw_crosshair(width: f32, height: f32) {
    draw_circle(width / 2.0, height / 2.0, 2.0, WHITE);
}

pub fn draw_selected_voxel(location: Location, camera: &Camera3D) {
    let position = vec3(
        location.x as f32 - camera.position.x,
        location.y as f32 - camera.position.y,
        location.z as f32 - camera.position.z,
    );
    draw_cube_wires(position, vec3(1.0, 1.0, 1.0), WHITE);
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct ItemHotbar {
    selected: usize,
    ui_size: f32,
}
impl ItemHotbar {
    pub fn new() -> Self {
        Self {
            selected: 0,
            ui_size: 0.05,
        }
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < Inventory::SELECTED_SIZE {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// draws the voxel selection ui
    pub fn draw(
        &self,
        items_on_hotbar: &[Option<Item>; Inventory::SELECTED_SIZE],
        asset_manager: &AssetManager,
    ) {
        let (screen_width, screen_height) = screen_size();
        let border_size = screen_width * self.ui_size;
        let picture_size = border_size * 0.8;
        let total_width = border_size * items_on_hotbar.len() as f32;
        let x_start = (screen_width - total_width) / 2.0;
        let y = screen_height - border_size;

        for (index, item) in items_on_hotbar.iter().enumerate() {
            let texture_with_count = item.as_ref().map(|non_empty| {
                (
                    asset_manager.texture_manager.get_icon(non_empty.voxel),
                    non_empty.count,
                )
            });
            let is_selected = self.selected == index;
            let x = x_start + index as f32 * border_size;

            Self::draw_voxel(
                border_size,
                picture_size,
                texture_with_count,
                x,
                y,
                is_selected,
                &asset_manager.font,
            );
        }
    }

    fn draw_voxel(
        border_size: f32,
        picture_size: f32,
        texture_with_count: Option<(Texture2D, u8)>,
        x: f32,
        y: f32,
        is_selected: bool,
        font: &Font,
    ) {
        let border_color = if is_selected {
            TEXT_COLOR
        } else {
            Color::from_rgba(120, 120, 120, 150)
        };
        let offset = (border_size - picture_size) / 2.0;

        draw_rectangle(x, y, border_size, border_size, border_color);
        if let Some((texture, count)) = texture_with_count {
            draw_texture_ex(
                &texture,
                x + offset,
                y + offset,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(picture_size, picture_size)),
                    ..Default::default()
                },
            );

            let font_size = BASE_COUNT_FONT_SIZE * border_size;
            use_str_buffer(|buffer| {
                write!(buffer, "{count}").expect("error writing to text buffer");
                draw_game_text(
                    buffer,
                    x + offset,
                    y + font_size * 1.7,
                    font_size,
                    TEXT_COLOR,
                    font,
                );
            });
        } else {
            draw_rectangle(
                x + offset,
                y + offset,
                picture_size,
                picture_size,
                Color::from_rgba(0, 0, 0, 100),
            );
        }
    }
}
