use std::{rc::Rc, thread::sleep, time::Duration};

use macroquad::time::get_frame_time;

use crate::{
    interface::interface_context::InterfaceContext, model::user_settings::UserSettings,
    service::asset_manager::AssetManager, voxel_engine::VoxelEngine,
};

pub enum GameState {
    Running { voxel_engine: Box<VoxelEngine> },
    Menu { context: Box<InterfaceContext> },
    Exit,
}
impl GameState {
    pub fn new(asset_manager: Rc<AssetManager>, user_settings: UserSettings) -> Self {
        Self::Menu {
            context: Box::new(InterfaceContext::new_title_screen(
                asset_manager,
                user_settings,
            )),
        }
    }

    /// returns false if the game should exit
    pub async fn process_next_frame(&mut self) -> bool {
        match self {
            GameState::Running { voxel_engine } => {
                if let Some(new_state) = Self::handle_running_state(voxel_engine).await {
                    *self = new_state;
                }
            }
            GameState::Menu { context } => {
                if let Some(new_state) = Self::handle_menu_state(context).await {
                    *self = new_state;
                }
            }
            GameState::Exit => {
                sleep(Duration::from_millis(200));
                return false;
            }
        }

        true
    }

    async fn handle_running_state(voxel_engine: &mut VoxelEngine) -> Option<GameState> {
        let delta = get_frame_time().min(0.1);
        let raycast_result = voxel_engine.process_input(delta);
        voxel_engine.update_loaded_areas();
        voxel_engine.update_processes(delta);
        voxel_engine.draw_scene(raycast_result).await
    }

    async fn handle_menu_state(context: &mut InterfaceContext) -> Option<GameState> {
        if let Some(mut voxel_engine) = context.enter_game() {
            voxel_engine.load_world();
            Some(GameState::Running { voxel_engine })
        } else if context.should_exit() {
            Some(GameState::Exit)
        } else {
            context.draw().await;
            None
        }
    }
}
