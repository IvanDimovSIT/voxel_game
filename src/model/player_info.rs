use macroquad::math::Vec3;

use crate::service::camera_controller::CameraController;

#[derive(Debug)]
pub struct PlayerInfo {
    pub camera_controller: CameraController,
    pub move_speed: f32,
    pub voxel_reach: f32,
    pub velocity: f32,
    pub jump_velocity: f32,
}
impl PlayerInfo {
    pub fn new(position: Vec3) -> Self {
        Self {
            camera_controller: CameraController::new(position),
            move_speed: 10.0,
            voxel_reach: 7.0,
            velocity: 0.0,
            jump_velocity: -0.15,
        }
    }
}
