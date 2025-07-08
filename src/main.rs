#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{rc::Rc, thread::sleep, time::Duration};

use graphics::texture_manager::TextureManager;
use macroquad::{
    conf::Conf, miniquad::window::set_fullscreen, texture::FilterMode, time::get_frame_time,
};
use model::user_settings::UserSettings;
use service::sound_manager::SoundManager;
use voxel_engine::VoxelEngine;

use crate::{
    interface::interface_context::InterfaceContext,
    service::persistence::user_settings_persistence::{
        read_or_initialise_user_settings, write_user_settings,
    },
};

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
    fn new(
        texture_manager: Rc<TextureManager>,
        sound_manager: Rc<SoundManager>,
        user_settings: UserSettings,
    ) -> Self {
        Self::Menu {
            context: Box::new(InterfaceContext::new_title_screen(
                sound_manager,
                texture_manager,
                user_settings,
            )),
        }
    }
}

#[macroquad::main("Voxel World", config)]
async fn main() {
    let texture_manager = Rc::new(TextureManager::new().await);
    let sound_manager = Rc::new(SoundManager::new().await);
    let user_settings = read_or_initialise_user_settings();
    if user_settings.is_fullscreen {
        set_fullscreen(true);
    }
    let mut state = GameState::new(
        texture_manager.clone(),
        sound_manager.clone(),
        user_settings,
    );

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
                if let Some(mut voxel_engine) = context.enter_game() {
                    voxel_engine.load_world();
                    state = GameState::Running { voxel_engine }
                } else if context.should_exit() {
                    state = GameState::Exit
                } else {
                    context.draw().await;
                }
            }
            GameState::Exit => {
                sleep(Duration::from_millis(200));
                break;
            }
        }
    }
}
