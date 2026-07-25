use bincode::{Decode, Encode};
use macroquad::math::Vec3;

use crate::{
    graphics::ui_display::ItemHotbar,
    model::inventory::Inventory,
    service::{activity_timer::ActivityTimer, camera_controller::CameraController},
    utils::{arr_to_vec3, vec3_to_arr},
};

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
    pub velocity: Vec3,
    pub water_status: PlayerWaterStatus,
}
impl PlayerInfo {
    pub const PLAYER_HEIGHT: f32 = 1.5;
    pub const PLAYER_MOVE_SPEED: f32 = 9.0;
    pub const PLAYER_SIZE: f32 = 0.3;
    pub const VOXEL_REACH: f32 = 7.0;
    pub const JUMP_VELOCITY: f32 = -15.0;

    pub fn new(position: Vec3) -> Self {
        Self {
            camera_controller: CameraController::new(position),
            velocity: Vec3::ZERO,
            voxel_selector: ItemHotbar::new(),
            inventory: Inventory::new_initial(),
            water_status: PlayerWaterStatus::Dry,
            destroy_progress: ActivityTimer::new(0.0, DESTROY_VOXEL_DELAY),
            place_progress: ActivityTimer::new(0.0, PLACE_VOXEL_DELAY),
            replace_progress: ActivityTimer::new(0.0, REPLACE_VOXEL_DELAY),
        }
    }

    pub fn create_dto(&self) -> PlayerInfoDTO {
        let position = self.camera_controller.get_position();
        PlayerInfoDTO {
            velocity: vec3_to_arr(self.velocity),
            position: vec3_to_arr(position),
            yaw: self.camera_controller.yaw,
            pitch: self.camera_controller.pitch,
            voxel_selector: self.voxel_selector.clone(),
            current_selection: self.voxel_selector.get_selected_index(),
            inventory: self.inventory.clone(),
        }
    }

    pub fn is_in_water(&self) -> bool {
        self.water_status != PlayerWaterStatus::Dry
    }
}
impl From<PlayerInfoDTO> for PlayerInfo {
    fn from(value: PlayerInfoDTO) -> Self {
        let position = arr_to_vec3(value.position);
        let velocity = arr_to_vec3(value.velocity);
        let mut camera_controller = CameraController::new(position);
        camera_controller.yaw = value.yaw;
        camera_controller.pitch = value.pitch;

        Self {
            camera_controller,
            velocity,
            voxel_selector: value.voxel_selector,
            inventory: value.inventory,
            water_status: PlayerWaterStatus::Dry,
            destroy_progress: ActivityTimer::new(0.0, DESTROY_VOXEL_DELAY),
            place_progress: ActivityTimer::new(0.0, PLACE_VOXEL_DELAY),
            replace_progress: ActivityTimer::new(0.0, REPLACE_VOXEL_DELAY),
        }
    }
}

#[derive(Debug, Clone, Copy, Encode, Decode, PartialEq, Eq)]
pub enum PlayerWaterStatus {
    Dry,
    PartiallyInWater,
    FullyInWater,
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct PlayerInfoDTO {
    inventory: Inventory,
    velocity: [f32; 3],
    position: [f32; 3],
    voxel_selector: ItemHotbar,
    current_selection: usize,
    yaw: f32,
    pitch: f32,
}
