use macroquad::{
    camera::set_default_camera,
    color::{BLACK, Color},
    math::Rect,
    miniquad::window::screen_size,
    shapes::{draw_rectangle, draw_rectangle_lines},
    window::set_fullscreen,
};

use crate::{
    graphics::screen_effects::darken_background,
    interface::{
        button::draw_button,
        game_menu::crafting_menu::CraftingMenuHandle,
        style::{BACKGROUND_COLOR, BUTTON_COLOR},
        text::draw_game_text,
    },
    model::{inventory::Item, user_settings::UserSettings},
    service::asset_manager::AssetManager,
};

const MENU_BOX_WIDTH: f32 = 400.0;
const MENU_BOX_HEIGHT: f32 = 400.0;
const BUTTON_WIDTH: f32 = 250.0;
const BUTTON_HEIGHT: f32 = 60.0;
const BUTTON_TEXT_SIZE: u16 = 30;

#[derive(Debug, Clone, Copy)]
pub enum MenuSelection {
    None,
    BackToGame,
    ToWorldSelection,
    ToOptions,
    ToMainMenu,
    Exit,
}

#[derive(Debug, Clone)]
pub enum MenuState {
    Hidden,
    Main,
    Options,
    ItemSelection {
        currently_selected_item: Option<Item>,
    },
    Crafting(CraftingMenuHandle),
}
impl MenuState {
    /// returns true if a menu is being displayed
    pub fn is_in_menu(&self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

/// draws the in game main menu
pub fn draw_main_menu(asset_manager: &AssetManager, user_settings: &UserSettings) -> MenuSelection {
    set_default_camera();
    let (width, height) = screen_size();
    darken_background(width, height);

    let (menu_x, menu_y) = calculate_menu_position(width, height);
    draw_menu_background(menu_x, menu_y);
    let button_x = menu_x + (MENU_BOX_WIDTH - BUTTON_WIDTH) * 0.5;
    let button_y_start = menu_y + 30.0;

    let is_back_to_game = draw_button(
        Rect {
            x: button_x,
            y: button_y_start,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        "Back to game",
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    );
    let is_to_world_selection = draw_button(
        Rect {
            x: button_x,
            y: button_y_start + BUTTON_HEIGHT * 1.5,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        "To world selection",
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    );
    let is_options = draw_button(
        Rect {
            x: button_x,
            y: button_y_start + BUTTON_HEIGHT * 3.0,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        "Options",
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    );
    let is_exit = draw_button(
        Rect {
            x: button_x,
            y: button_y_start + BUTTON_HEIGHT * 4.5,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        "Exit game",
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    );

    if is_exit {
        MenuSelection::Exit
    } else if is_back_to_game {
        MenuSelection::BackToGame
    } else if is_to_world_selection {
        MenuSelection::ToWorldSelection
    } else if is_options {
        MenuSelection::ToOptions
    } else {
        MenuSelection::None
    }
}

/// draws the in game options menu
/// callback forces blocking area mesh generation
pub fn draw_options_menu<F: FnMut(&UserSettings)>(
    asset_manager: &AssetManager,
    user_settings: &mut UserSettings,
    mut change_render_callback: F,
) -> MenuSelection {
    set_default_camera();
    let (width, height) = screen_size();
    darken_background(width, height);

    let (menu_x, menu_y) = calculate_menu_position(width, height);
    draw_menu_background(menu_x, menu_y);
    let contents_x = menu_x + (MENU_BOX_WIDTH - BUTTON_WIDTH) * 0.5;
    let contents_y = menu_y + 30.0;

    let render_distance_button_width = 50.0;
    let render_distance_button_height = 50.0;
    let render_distance_display_width = 220.0;
    let render_distance_x = menu_x
        + (MENU_BOX_WIDTH - render_distance_button_width * 2.0 - render_distance_display_width)
            * 0.5;
    let decrese_render_distance = draw_button(
        Rect {
            x: render_distance_x,
            y: contents_y,
            w: render_distance_button_width,
            h: render_distance_button_height,
        },
        "-",
        40,
        asset_manager,
        user_settings,
    );
    draw_game_text(
        &format!("View distance {}", user_settings.get_render_distance()),
        render_distance_x + render_distance_button_width + 10.0,
        contents_y + render_distance_button_height * 0.8,
        28.0,
        BUTTON_COLOR,
        &asset_manager.font,
    );
    let increase_render_distance = draw_button(
        Rect {
            x: render_distance_x + render_distance_button_width + render_distance_display_width,
            y: contents_y,
            w: render_distance_button_width,
            h: render_distance_button_height,
        },
        "+",
        40,
        asset_manager,
        user_settings,
    );
    let toggle_sounds =
        draw_toggle_sound_button(asset_manager, user_settings, contents_x, contents_y);
    let toggle_fullscreen =
        draw_toggle_fullscreen_button(asset_manager, user_settings, contents_x, contents_y);
    let should_go_back = draw_go_back_button(asset_manager, user_settings, contents_x, contents_y);

    if toggle_fullscreen {
        user_settings.is_fullscreen = !user_settings.is_fullscreen;
        set_fullscreen(user_settings.is_fullscreen);
    }
    if toggle_sounds {
        user_settings.has_sound = !user_settings.has_sound;
    }
    if increase_render_distance {
        let _increased = user_settings.increase_render_distance();
        change_render_callback(user_settings);
    }
    if decrese_render_distance {
        let _decreased = user_settings.decrease_render_distance();
        change_render_callback(user_settings);
    }

    if should_go_back {
        MenuSelection::ToMainMenu
    } else {
        MenuSelection::None
    }
}

/// returns true if pressed
fn draw_toggle_sound_button(
    asset_manager: &AssetManager,
    user_settings: &mut UserSettings,
    contents_x: f32,
    contents_y: f32,
) -> bool {
    draw_button(
        Rect {
            x: contents_x,
            y: contents_y + BUTTON_HEIGHT * 1.5,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        if user_settings.has_sound {
            "Sound:ON"
        } else {
            "Sound:OFF"
        },
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    )
}

/// returns true if pressed
fn draw_toggle_fullscreen_button(
    asset_manager: &AssetManager,
    user_settings: &mut UserSettings,
    contents_x: f32,
    contents_y: f32,
) -> bool {
    draw_button(
        Rect {
            x: contents_x,
            y: contents_y + BUTTON_HEIGHT * 3.0,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        if user_settings.is_fullscreen {
            "Go windowed"
        } else {
            "Go fullscreen"
        },
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    )
}

/// returns true if pressed
fn draw_go_back_button(
    asset_manager: &AssetManager,
    user_settings: &mut UserSettings,
    contents_x: f32,
    contents_y: f32,
) -> bool {
    draw_button(
        Rect {
            x: contents_x,
            y: contents_y + BUTTON_HEIGHT * 4.5,
            w: BUTTON_WIDTH,
            h: BUTTON_HEIGHT,
        },
        "Back",
        BUTTON_TEXT_SIZE,
        asset_manager,
        user_settings,
    )
}

fn calculate_menu_position(width: f32, height: f32) -> (f32, f32) {
    let menu_x = (width - (MENU_BOX_WIDTH)) * 0.5;
    let menu_y = height * 0.1;
    (menu_x, menu_y)
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
