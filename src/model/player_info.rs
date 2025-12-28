use bincode::{Decode, Encode};
use macroquad::math::Vec3;

use crate::{
    graphics::ui_display::ItemHotbar,
    model::inventory::Inventory,
    service::{activity_timer::ActivityTimer, camera_controller::CameraController},
    utils::{arr_to_vec3, vec3_to_arr},
};

const PLAYER_MOVE_SPEED: f32 = 9.0;
const PLAYER_SIZE: f32 = 0.3;
const VOXEL_REACH: f32 = 7.0;
const JUMP_VELOCITY: f32 = -15.0;
const DESTROY_VOXEL_DELAY: f32 = 0.25;
const PLACE_VOXEL_DELAY: f32 = 0.2;
const REPLACE_VOXEL_DELAY: f32 = 0.1;

#[derive(Debug)]
pub struct PlayerInfo {
    pub destroy_progress: ActivityTimer,
    pub place_progress: ActivityTimer,
    pub replace_progress: ActivityTimer,
    pub inventory: Inventory,
    pub camera_controller: CameraController,
    pub voxel_selector: ItemHotbar,
    pub move_speed: f32,
    pub voxel_reach: f32,
    pub size: f32,
    pub velocity: f32,
    pub jump_velocity: f32,
    pub is_in_water: bool,
    pub is_head_in_water: bool,
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
            voxel_selector: ItemHotbar::new(),
            inventory: Inventory::default(),
            is_in_water: false,
            is_head_in_water: false,
            destroy_progress: ActivityTimer::new(0.0, DESTROY_VOXEL_DELAY),
            place_progress: ActivityTimer::new(0.0, PLACE_VOXEL_DELAY),
            replace_progress: ActivityTimer::new(0.0, REPLACE_VOXEL_DELAY),
        }
    }

    pub fn create_dto(&self) -> PlayerInfoDTO {
        let position = self.camera_controller.get_position();
        PlayerInfoDTO {
            velocity: self.velocity,
            position: vec3_to_arr(position),
            yaw: self.camera_controller.yaw,
            pitch: self.camera_controller.pitch,
            voxel_selector: self.voxel_selector.clone(),
            current_selection: self.voxel_selector.get_selected_index(),
            inventory: self.inventory.clone(),
        }
    }
}
impl From<PlayerInfoDTO> for PlayerInfo {
    fn from(value: PlayerInfoDTO) -> Self {
        let position = arr_to_vec3(value.position);
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
            voxel_selector: value.voxel_selector,
            inventory: value.inventory,
            is_in_water: false,
            is_head_in_water: false,
            destroy_progress: ActivityTimer::new(0.0, DESTROY_VOXEL_DELAY),
            place_progress: ActivityTimer::new(0.0, PLACE_VOXEL_DELAY),
            replace_progress: ActivityTimer::new(0.0, REPLACE_VOXEL_DELAY),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInfoDTO {
    inventory: Inventory,
    velocity: f32,
    position: [f32; 3],
    voxel_selector: ItemHotbar,
    current_selection: usize,
    yaw: f32,
    pitch: f32,
}
