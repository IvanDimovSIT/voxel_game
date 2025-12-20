#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use macroquad::{
    miniquad::{conf::Icon, window::set_fullscreen},
    window::Conf,
};

use crate::{
    game_state::GameState,
    service::{
        asset_manager::AssetManager,
        persistence::{
            generic_persistence::initialise_save_directory,
            user_settings_persistence::read_or_initialise_user_settings,
        },
    },
};

mod game_state;
mod graphics;
mod interface;
mod model;
mod service;
mod utils;
mod voxel_engine;

include!(concat!(env!("OUT_DIR"), "/icons.rs"));

fn config() -> Conf {
    Conf {
        window_title: "Voxel World".to_owned(),
        icon: Some(Icon {
            small: SMALL_ICON,
            medium: MEDIUM_ICON,
            big: LARGE_ICON,
        }),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    initialise_save_directory();
    let asset_manager_result = AssetManager::new().await;
    let user_settings = read_or_initialise_user_settings();
    if user_settings.is_fullscreen {
        set_fullscreen(true);
    }

    let mut state = match asset_manager_result {
        Ok(asset_manager) => GameState::new(asset_manager.clone(), user_settings),
        Err(errors) => GameState::error(errors),
    };

    loop {
        if !state.process_next_frame().await {
            break;
        }
    }
}
