use interface::InterfaceContext;
use macroquad::{conf::Conf, texture::FilterMode, time::get_frame_time};
use voxel_engine::VoxelEngine;

mod graphics;
mod model;
mod service;
mod utils;
mod voxel_engine;
mod interface;

fn config() -> Conf {
    Conf {
        draw_call_vertex_capacity: 50,
        draw_call_index_capacity: 50,
        default_filter_mode: FilterMode::Nearest,
        ..Default::default()
    }
}

enum GameState {
    Running {voxel_engine: Box<VoxelEngine>},
    Menu {context: Box<InterfaceContext>}
}
impl Default for GameState {
    fn default() -> Self {
        Self::Menu { context: Box::new(InterfaceContext::new()) }
    }
}

#[macroquad::main("Voxel World", config)]
async fn main() {
    let mut state = GameState::default();

    loop {
        match &mut state {
            GameState::Running { voxel_engine } => {
                let delta = get_frame_time().min(0.2);
                let raycast_result = voxel_engine.process_input(delta);
                voxel_engine.process_physics(delta);
                voxel_engine.update_loaded_areas();
                voxel_engine.draw_scene(raycast_result).await;
            },
            GameState::Menu { context } => {
                if let Some(voxel_engine) = context.enter_game().await {
                    state = GameState::Running { voxel_engine }
                } else {
                    context.draw().await;
                }
            }
        }
    }
}
