use macroquad::{
    camera::set_default_camera,
    color::{BLACK, Color},
    input::{KeyCode, is_key_pressed, is_key_released},
    miniquad::window::screen_size,
    shapes::{draw_rectangle, draw_rectangle_lines},
};

use crate::service::sound_manager::SoundManager;

use super::{button::draw_button, style::*};

const CLEAR_SCREEN_COLOR: Color = Color::from_rgba(0, 0, 0, 100);

const MENU_BOX_WIDTH: f32 = 400.0;
const MENU_BOX_HEIGHT: f32 = 300.0;

#[derive(Debug, Clone, Copy)]
pub enum MenuSelection {
    None,
    BackToGame,
    ToWorldSelection,
    Exit,
}

pub fn draw_menu(sound_manager: &SoundManager) -> MenuSelection {
    set_default_camera();
    let (width, height) = screen_size();
    darken_background(width, height);

    let menu_x = (width - (MENU_BOX_WIDTH)) * 0.5;
    let menu_y = height * 0.1;
    draw_menu_background(menu_x, menu_y);
    let button_width = 250.0;
    let button_height = 60.0;
    let text_size = 30;
    let button_x = menu_x + (MENU_BOX_WIDTH - button_width) * 0.5;
    let button_y_start = menu_y + 30.0;

    let is_back_to_game = draw_button(
        button_x,
        button_y_start,
        button_width,
        button_height,
        "Back to game",
        text_size,
        sound_manager,
    );
    let is_to_world_selection = draw_button(
        button_x,
        button_y_start + button_height * 1.5,
        button_width,
        button_height,
        "To world selection",
        text_size,
        sound_manager,
    );
    let is_exit = draw_button(
        button_x,
        button_y_start + button_height * 3.0,
        button_width,
        button_height,
        "Exit game",
        text_size,
        sound_manager,
    );

    if is_exit {
        MenuSelection::Exit
    } else if is_back_to_game {
        MenuSelection::BackToGame
    } else if is_to_world_selection {
        MenuSelection::ToWorldSelection
    } else {
        MenuSelection::None
    }
}

fn darken_background(width: f32, height: f32) {
    draw_rectangle(0.0, 0.0, width, height, CLEAR_SCREEN_COLOR);
}

fn draw_menu_background(menu_x: f32, menu_y: f32) {
    draw_rectangle(
        menu_x - 3.0,
        menu_y + 3.0,
        MENU_BOX_WIDTH,
        MENU_BOX_HEIGHT,
        Color::from_rgba(0, 0, 0, 150),
    );
    draw_rectangle(
        menu_x,
        menu_y,
        MENU_BOX_WIDTH,
        MENU_BOX_HEIGHT,
        BACKGROUND_COLOR,
    );
    draw_rectangle_lines(menu_x, menu_y, MENU_BOX_WIDTH, MENU_BOX_HEIGHT, 3.0, BLACK);
}
