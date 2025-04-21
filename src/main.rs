use std::rc::Rc;

use graphics::texture_manager::TextureManager;
use interface::InterfaceContext;
use macroquad::{conf::Conf, texture::FilterMode, time::get_frame_time};
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
        draw_call_vertex_capacity: 50,
        draw_call_index_capacity: 50,
        default_filter_mode: FilterMode::Nearest,
        ..Default::default()
    }
}

enum GameState {
    Running { voxel_engine: Box<VoxelEngine> },
    Menu { context: Box<InterfaceContext> },
    Exit,
}
impl Default for GameState {
    fn default() -> Self {
        Self::Menu {
            context: Box::new(InterfaceContext::new()),
        }
    }
}

#[macroquad::main("Voxel World", config)]
async fn main() {
    let texture_manager = Rc::new(TextureManager::new().await);
    let sound_manager = Rc::new(SoundManager::new().await);
    let mut state = GameState::default();

    loop {
        match &mut state {
            GameState::Running { voxel_engine } => {
                let delta = get_frame_time().min(0.2);
                let raycast_result = voxel_engine.process_input(delta);
                voxel_engine.process_physics(delta);
                voxel_engine.update_loaded_areas();
                let change_context = voxel_engine.draw_scene(raycast_result).await;
                if let Some(new_context) = change_context {
                    state = new_context
                }
            }
            GameState::Menu { context } => {
                if let Some(voxel_engine) = context
                    .enter_game(texture_manager.clone(), sound_manager.clone())
                    .await
                {
                    state = GameState::Running { voxel_engine }
                } else {
                    context.draw().await;
                }
            }
            GameState::Exit => break,
        }
    }
}
