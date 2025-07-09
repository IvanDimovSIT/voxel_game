use bincode::{Decode, Encode};
use macroquad::math::{Vec3, vec3};

use crate::{
    graphics::ui_display::{VOXEL_SELECTION_SIZE, VoxelSelector},
    model::voxel::Voxel,
    service::camera_controller::CameraController,
};

const PLAYER_MOVE_SPEED: f32 = 10.0;
const PLAYER_SIZE: f32 = 0.3;
const VOXEL_REACH: f32 = 7.0;
const JUMP_VELOCITY: f32 = -15.0;

#[derive(Debug)]
pub struct PlayerInfo {
    pub camera_controller: CameraController,
    pub voxel_selector: VoxelSelector,
    pub move_speed: f32,
    pub voxel_reach: f32,
    pub size: f32,
    pub velocity: f32,
    pub jump_velocity: f32,
}
impl PlayerInfo {
    pub fn new(position: Vec3) -> Self {
        Self {
            camera_controller: CameraController::new(position),
            move_speed: PLAYER_MOVE_SPEED,
            voxel_reach: VOXEL_REACH,
            velocity: 0.0,
            jump_velocity: JUMP_VELOCITY,
            size: PLAYER_SIZE,
            voxel_selector: VoxelSelector::new(),
        }
    }

    pub fn create_dto(&self) -> PlayerInfoDTO {
        let position = self.camera_controller.get_position();
        PlayerInfoDTO {
            velocity: self.velocity,
            position: [position.x, position.y, position.z],
            yaw: self.camera_controller.yaw,
            pitch: self.camera_controller.pitch,
            voxel_selection: self.voxel_selector.get_voxels(),
            current_selection: self.voxel_selector.get_selected_index(),
        }
    }
}
impl From<PlayerInfoDTO> for PlayerInfo {
    fn from(value: PlayerInfoDTO) -> Self {
        let position = vec3(value.position[0], value.position[1], value.position[2]);
        let mut camera_controller = CameraController::new(position);
        camera_controller.yaw = value.yaw;
        camera_controller.pitch = value.pitch;

        Self {
            camera_controller,
            move_speed: PLAYER_MOVE_SPEED,
            voxel_reach: VOXEL_REACH,
            velocity: value.velocity,
            jump_velocity: JUMP_VELOCITY,
            size: PLAYER_SIZE,
            voxel_selector: VoxelSelector::from_saved(
                value.voxel_selection,
                value.current_selection,
            ),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInfoDTO {
    velocity: f32,
    position: [f32; 3],
    voxel_selection: [Option<Voxel>; VOXEL_SELECTION_SIZE],
    current_selection: usize,
    yaw: f32,
    pitch: f32,
}
