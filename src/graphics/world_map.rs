use std::f32::consts::{PI, TAU};

use macroquad::{
    camera::Camera3D,
    color::WHITE,
    math::{Vec2, Vec3, vec3},
    texture::{DrawTextureParams, draw_texture_ex},
    window::{clear_background, screen_height},
};

use crate::{
    graphics::texture_manager::{PlainTextureId, TextureManager},
    interface::style::BACKGROUND_COLOR,
    model::{area::AREA_HEIGHT, player_info::PlayerInfo},
};

const MIN_ZOOM: f32 = 0.001;
const MAX_ZOOM: f32 = 20.0;
const CHANGE_ZOOM: f32 = 22.0;
const CHANGE_LEFT_RIGHT_ANGLE: f32 = 1.0;
const CHANGE_UP_DOWN_ANGLE: f32 = 1.2;
const MIN_UP_DOWN_ANGLE: f32 = PI * 1.45;
const MAX_UP_DOWN_ANGLE: f32 = PI * 1.95;

const WORLD_UP: Vec3 = vec3(0.0, 0.0, -1.0);
const BASE_CAMERA_HEIGHT: f32 = -120.0;

#[derive(Debug)]
pub struct WorldMap {
    zoom_level: f32,
    pub active: bool,
    up_down_angle: f32,
    left_right_angle: f32,
}
impl WorldMap {
    pub fn new() -> Self {
        const DEFAULT_UP_DOWN_ANGLE: f32 = (MIN_UP_DOWN_ANGLE + MAX_UP_DOWN_ANGLE) / 2.0;

        Self {
            zoom_level: 1.0,
            active: false,
            up_down_angle: DEFAULT_UP_DOWN_ANGLE,
            left_right_angle: PI / 2.0,
        }
    }

    pub fn increase_zoom(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level + delta * CHANGE_ZOOM).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub fn decrease_zoom(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level - delta * CHANGE_ZOOM).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub fn increase_up_down_angle(&mut self, delta: f32) {
        self.up_down_angle = (self.up_down_angle + delta * CHANGE_UP_DOWN_ANGLE)
            .clamp(MIN_UP_DOWN_ANGLE, MAX_UP_DOWN_ANGLE);
    }

    pub fn decrease_up_down_angle(&mut self, delta: f32) {
        self.up_down_angle = (self.up_down_angle - delta * CHANGE_UP_DOWN_ANGLE)
            .clamp(MIN_UP_DOWN_ANGLE, MAX_UP_DOWN_ANGLE);
    }

    pub fn increase_left_right_angle(&mut self, delta: f32) {
        self.left_right_angle =
            (self.left_right_angle + delta * CHANGE_LEFT_RIGHT_ANGLE).rem_euclid(TAU);
    }

    pub fn decrease_left_right_angle(&mut self, delta: f32) {
        self.left_right_angle =
            (self.left_right_angle - delta * CHANGE_LEFT_RIGHT_ANGLE).rem_euclid(TAU);
    }

    pub fn create_map_camera(&self, player_info: &PlayerInfo) -> Camera3D {
        assert!(self.active);

        const Z_DISTANCE: f32 = 60.0;
        const WORLD_VIEW_SIZE: f32 = 100.0;
        const BASE_MAP_FOV: f32 = 0.7;
        const MAP_FOV_MOD: f32 = 0.07;
        const TARGET_Z: f32 = (AREA_HEIGHT / 2) as f32;

        let map_fov = BASE_MAP_FOV + self.zoom_level * MAP_FOV_MOD;

        let player_pos = player_info.camera_controller.get_position();
        let player_x = player_pos.x;
        let player_y = player_pos.y;

        let x_offset = Z_DISTANCE * self.left_right_angle.cos() * self.up_down_angle.sin();
        let y_offset = Z_DISTANCE * self.left_right_angle.sin() * self.up_down_angle.sin();

        let camera_pos = vec3(
            player_x + x_offset,
            player_y + y_offset,
            BASE_CAMERA_HEIGHT * self.up_down_angle.cos(),
        );

        let target_pos = vec3(player_x, player_y, TARGET_Z);

        let front = (target_pos - camera_pos).normalize();
        let right = front.cross(WORLD_UP).normalize();
        let up = right.cross(front).normalize();

        Camera3D {
            position: camera_pos,
            target: target_pos,
            up,
            projection: macroquad::camera::Projection::Orthographics,
            fovy: WORLD_VIEW_SIZE * map_fov,
            ..Default::default()
        }
    }

    /// draws the compass 2D visual
    pub fn draw_comapass(&self, texture_manager: &TextureManager) {
        const COMPASS_RELATIVE_SIZE: f32 = 0.15;
        let rotation = 0.5 * PI - self.left_right_angle;
        let height = screen_height();
        let compass_size = COMPASS_RELATIVE_SIZE * height;
        let x = 0.0;
        let y = height - compass_size;
        let compass_texture = texture_manager.get_plain_texture(PlainTextureId::Compass);
        let params = DrawTextureParams {
            dest_size: Some(Vec2::splat(compass_size)),
            rotation,
            ..Default::default()
        };

        draw_texture_ex(&compass_texture, x, y, WHITE, params);
    }

    pub fn draw_background(&self) {
        assert!(self.active);
        clear_background(BACKGROUND_COLOR);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_increase_zoom() {
        let mut map = WorldMap::new();
        let zoom1 = map.zoom_level;

        map.increase_zoom(10.0);
        let zoom2 = map.zoom_level;

        assert!(zoom1 < zoom2);

        map.increase_zoom(10.0);
        let zoom3 = map.zoom_level;

        assert!(zoom2 == zoom3);
    }

    #[test]
    fn test_decrease_zoom() {
        let mut map = WorldMap::new();
        let zoom1 = map.zoom_level;

        map.decrease_zoom(10.0);
        let zoom2 = map.zoom_level;

        assert!(zoom1 > zoom2);

        map.decrease_zoom(10.0);
        let zoom3 = map.zoom_level;

        assert!(zoom2 == zoom3);
    }

    #[test]
    fn test_increase_up_down_angle() {
        let mut map = WorldMap::new();
        map.up_down_angle = MIN_UP_DOWN_ANGLE + 1.0;
        let angle1 = map.up_down_angle;

        map.increase_up_down_angle(0.01);
        let angle2 = map.up_down_angle;

        assert!(angle1 < angle2);

        map.increase_up_down_angle(10.0);
        let angle3 = map.up_down_angle;

        assert!((angle3 - MAX_UP_DOWN_ANGLE).abs() < std::f32::EPSILON);
    }

    #[test]
    fn test_decrease_up_down_angle() {
        let mut map = WorldMap::new();
        map.up_down_angle = MAX_UP_DOWN_ANGLE - 1.0;
        let angle1 = map.up_down_angle;

        map.decrease_up_down_angle(0.01);
        let angle2 = map.up_down_angle;

        assert!(angle1 > angle2);

        map.decrease_up_down_angle(10.0);
        let angle3 = map.up_down_angle;

        assert!((angle3 - MIN_UP_DOWN_ANGLE).abs() < std::f32::EPSILON);
    }

    #[test]
    fn test_increase_left_right_angle() {
        let mut map = WorldMap::new();
        map.left_right_angle = 0.0;
        let angle1 = map.left_right_angle;

        map.increase_left_right_angle(0.1);
        let angle2 = map.left_right_angle;

        assert!(angle1 < angle2);

        map.left_right_angle = TAU - 0.1;
        map.increase_left_right_angle(0.5);
        let angle3 = map.left_right_angle;

        assert!(angle3 < 0.5);
        assert!(angle3 > std::f32::EPSILON);
    }

    #[test]
    fn test_decrease_left_right_angle() {
        let mut map = WorldMap::new();
        map.left_right_angle = PI;
        let angle1 = map.left_right_angle;

        map.decrease_left_right_angle(0.1);
        let angle2 = map.left_right_angle;

        assert!(angle1 > angle2);

        map.left_right_angle = 0.1;
        map.decrease_left_right_angle(0.5);
        let angle3 = map.left_right_angle;

        assert!(angle3 > TAU - 1.0);
        assert!(angle3 < TAU);
    }
}
