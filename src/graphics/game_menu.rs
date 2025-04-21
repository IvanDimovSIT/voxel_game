use macroquad::{
    camera::set_default_camera,
    color::{BLACK, Color, LIGHTGRAY, ORANGE, WHITE},
    input::{is_mouse_button_released, mouse_position},
    miniquad::window::screen_size,
    shapes::{draw_rectangle, draw_rectangle_lines},
    text::{TextParams, draw_text_ex},
};

const CLEAR_SCREEN_COLOR: Color = Color::from_rgba(0, 0, 0, 100);

const MENU_BOX_WIDTH: f32 = 400.0;
const MENU_BOX_HEIGHT: f32 = 300.0;
const MENU_COLOR: Color = ORANGE;

const BUTTON_COLOR: Color = WHITE;
const BUTTON_HOVER_COLOR: Color = LIGHTGRAY;

pub enum MenuSelection {
    None,
    BackToGame,
    ToWorldSelection,
    Exit,
}

pub fn draw_menu() -> MenuSelection {
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
    );
    let is_to_world_selection = draw_button(
        button_x,
        button_y_start + button_height * 1.5,
        button_width,
        button_height,
        "To world selection",
        text_size,
    );
    let is_exit = draw_button(
        button_x,
        button_y_start + button_height * 3.0,
        button_width,
        button_height,
        "Exit game",
        text_size,
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
    draw_rectangle(menu_x, menu_y, MENU_BOX_WIDTH, MENU_BOX_HEIGHT, MENU_COLOR);
    draw_rectangle_lines(menu_x, menu_y, MENU_BOX_WIDTH, MENU_BOX_HEIGHT, 3.0, BLACK);
}

fn draw_button(x: f32, y: f32, w: f32, h: f32, text: &str, text_size: u16) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let is_hovered = (x..(x + w)).contains(&mouse_x) && (y..(y + h)).contains(&mouse_y);
    let button_color = if is_hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
    };

    draw_rectangle(x, y, w, h, button_color);
    draw_rectangle_lines(x, y, w, h, 2.0, BLACK);
    draw_text_ex(
        text,
        x + 5.0,
        y + h * 0.5 + text_size as f32 * 0.5,
        TextParams {
            font_size: text_size,
            color: BLACK,
            ..Default::default()
        },
    );

    is_hovered && is_mouse_button_released(macroquad::input::MouseButton::Left)
}
