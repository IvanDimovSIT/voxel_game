use std::rc::Rc;

use graphics::texture_manager::TextureManager;
use interface::world_selection::InterfaceContext;
use macroquad::{conf::Conf, texture::FilterMode, time::get_frame_time};
use model::user_settings::UserSettings;
use service::sound_manager::SoundManager;
use voxel_engine::VoxelEngine;

mod graphics;
mod interface;
mod model;
mod service;
mod utils;
mod voxel_engine;

fn config() -> Conf {
    Conf {
        default_filter_mode: FilterMode::Nearest,
        ..Default::default()
    }
}

enum GameState {
    Running { voxel_engine: Box<VoxelEngine> },
    Menu { context: Box<InterfaceContext> },
    Exit,
}
impl GameState {
    fn new(sound_manager: Rc<SoundManager>, user_settings: UserSettings) -> Self {
        Self::Menu {
            context: Box::new(InterfaceContext::new(sound_manager, user_settings)),
        }
    }
}

#[macroquad::main("Voxel World", config)]
async fn main() {
    let texture_manager = Rc::new(TextureManager::new().await);
    let sound_manager = Rc::new(SoundManager::new().await);
    let user_settings = UserSettings::default();
    let mut state = GameState::new(sound_manager.clone(), user_settings);

    loop {
        match &mut state {
            GameState::Running { voxel_engine } => {
                let delta = get_frame_time().min(0.2);
                let raycast_result = voxel_engine.process_input(delta);
                voxel_engine.update_processes(delta);
                voxel_engine.update_loaded_areas();
                let change_context = voxel_engine.draw_scene(raycast_result).await;
                if let Some(new_context) = change_context {
                    state = new_context
                }
            }
            GameState::Menu { context } => {
                if let Some(mut voxel_engine) =
                    context.enter_game(texture_manager.clone(), sound_manager.clone())
                {
                    voxel_engine.load_world();
                    state = GameState::Running { voxel_engine }
                } else {
                    context.draw().await;
                }
            }
            GameState::Exit => break,
        }
    }
}
