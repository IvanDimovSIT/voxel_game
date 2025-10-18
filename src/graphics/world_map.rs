use macroquad::{camera::Camera3D, math::vec3};

use crate::model::player_info::PlayerInfo;

const MIN_ZOOM: f32 = 0.001;
const MAX_ZOOM: f32 = 20.0;
const CHANGE_ZOOM: f32 = 15.0;

#[derive(Debug)]
pub struct WorldMap {
    zoom_level: f32,
    pub active: bool,
}
impl WorldMap {
    pub fn new() -> Self {
        Self {
            zoom_level: 1.0,
            active: false,
        }
    }

    pub fn increase_zoom(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level + delta * CHANGE_ZOOM).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    pub fn decrease_zoom(&mut self, delta: f32) {
        self.zoom_level = (self.zoom_level - delta * CHANGE_ZOOM).clamp(MIN_ZOOM, MAX_ZOOM);
    }

    /// create a top-down camera
    pub fn create_map_camera(&self, player_info: &PlayerInfo) -> Camera3D {
        assert!(self.active);

        const CAMERA_HEIGHT: f32 = -120.0;
        const Z_DISTANCE: f32 = 20.0;
        const ANGLE_FACTOR: f32 = std::f32::consts::FRAC_1_SQRT_2;
        const WORLD_VIEW_SIZE: f32 = 100.0;
        const BASE_MAP_FOV: f32 = 0.7;
        const MAP_FOV_MOD: f32 = 0.07;

        let map_fov = BASE_MAP_FOV + self.zoom_level * MAP_FOV_MOD;

        let player_pos = player_info.camera_controller.get_position();
        let player_x = player_pos.x;
        let player_y = player_pos.y;

        let target_pos = vec3(player_x, player_y, player_pos.z - Z_DISTANCE);

        let x_offset = CAMERA_HEIGHT * ANGLE_FACTOR;
        let y_offset = CAMERA_HEIGHT * ANGLE_FACTOR;

        let compensation_factor = 0.5;
        let comp_x = WORLD_VIEW_SIZE * compensation_factor * ANGLE_FACTOR;
        let comp_y = WORLD_VIEW_SIZE * compensation_factor * ANGLE_FACTOR;

        let camera_pos = vec3(
            player_x + comp_x + x_offset,
            player_y + comp_y + y_offset,
            player_pos.z + CAMERA_HEIGHT,
        );

        let front = (target_pos - camera_pos).normalize();
        let world_up = vec3(0.0, 0.0, -1.0);
        let right = front.cross(world_up).normalize();
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
}
