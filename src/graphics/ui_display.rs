use macroquad::{
    camera::Camera3D,
    color::{Color, WHITE},
    math::{vec2, vec3},
    miniquad::window::screen_size,
    models::draw_cube_wires,
    prelude::error,
    shapes::{draw_circle, draw_rectangle},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
};

use crate::model::{location::Location, voxel::Voxel};

use super::texture_manager::TextureManager;

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

pub const VOXEL_SELECTION_SIZE: usize = 8;

#[derive(Debug)]
pub struct VoxelSelector {
    voxels: [Option<Voxel>; VOXEL_SELECTION_SIZE],
    selected: usize,
    ui_size: f32,
}
impl VoxelSelector {
    pub fn new() -> Self {
        Self {
            voxels: [
                Some(Voxel::Brick),
                Some(Voxel::Boards),
                Some(Voxel::Stone),
                Some(Voxel::Sand),
                Some(Voxel::Dirt),
                Some(Voxel::Grass),
                Some(Voxel::Wood),
                Some(Voxel::Leaves),
            ],
            selected: 0,
            ui_size: 0.05,
        }
    }

    pub fn from_saved(voxels: [Option<Voxel>; VOXEL_SELECTION_SIZE], selected: usize) -> Self {
        if selected >= voxels.len() {
            error!("Invalid selected index for voxel selector: {}", selected);
            Self::new()
        } else {
            Self {
                voxels,
                selected,
                ui_size: 0.05,
            }
        }
    }

    pub fn get_voxels(&self) -> [Option<Voxel>; VOXEL_SELECTION_SIZE] {
        self.voxels
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.voxels.len() {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn get_selected(&self) -> Option<Voxel> {
        self.voxels[self.selected]
    }

    pub fn get_at(&self, index: usize) -> Option<Voxel> {
        if index < self.voxels.len() {
            self.voxels[index]
        } else {
            error!("Entered invalid voxel selection index: {}", index);
            None
        }
    }

    pub fn set_at(&mut self, index: usize, voxel: Option<Voxel>) {
        if index < self.voxels.len() {
            self.voxels[index] = voxel;
        } else {
            error!("Entered invalid voxel selection index: {}", index);
        }
    }

    /// draws the voxel selection ui
    pub fn draw(&self, texture_manager: &TextureManager) {
        let (screen_width, screen_height) = screen_size();
        let border_size = screen_width * self.ui_size;
        let picture_size = border_size * 0.8;
        let total_width = border_size * self.voxels.len() as f32;
        let x_start = (screen_width - total_width) / 2.0;
        let y = screen_height - border_size;

        for (index, voxel) in self.voxels.iter().enumerate() {
            let texture = if let Some(non_empty) = voxel {
                Some(texture_manager.get(*non_empty))
            } else {
                None
            };
            let is_selected = self.selected == index;
            let x = x_start + index as f32 * border_size;

            Self::draw_voxel(border_size, picture_size, texture, x, y, is_selected);
        }
    }

    fn draw_voxel(
        border_size: f32,
        picture_size: f32,
        texture: Option<Texture2D>,
        x: f32,
        y: f32,
        is_selected: bool,
    ) {
        let border_color = if is_selected {
            WHITE
        } else {
            Color::from_rgba(120, 120, 120, 150)
        };
        let offset = (border_size - picture_size) / 2.0;

        draw_rectangle(x, y, border_size, border_size, border_color);
        if let Some(some_texture) = texture {
            draw_texture_ex(
                &some_texture,
                x + offset,
                y + offset,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(picture_size, picture_size)),
                    ..Default::default()
                },
            );
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
