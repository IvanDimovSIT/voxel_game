use macroquad::{
    camera::set_default_camera,
    input::mouse_position,
    math::Rect,
    miniquad::window::{screen_size, set_fullscreen},
    text::Font,
    window::next_frame,
};

use crate::{
    interface::{
        background::draw_background,
        button::{draw_back_button, draw_button},
        interface_context::InterfaceScreen,
        style::TEXT_COLOR,
        text::{draw_centered_multiline_text, draw_game_text, draw_version_number, get_text_width},
        title_screen::TitleScreenContext,
        util::is_point_in_rect,
    },
    model::user_settings::{ShadowType, UserSettings},
    service::{
        asset_manager::AssetManager, persistence::user_settings_persistence::write_user_settings,
    },
};

const BUTTON_WIDTH: f32 = 380.0;
const BUTTON_HEIGHT: f32 = 70.0;
const BUTTON_HEIGHT_OFFSET: f32 = BUTTON_HEIGHT * 1.2;
const BUTTON_TEXT_SIZE: f32 = 30.0;
const RENDER_DISTANCE_TEXT_WIDTH: f32 = 320.0;
const SETTINGS_TEXT_WIDTH: f32 = 80.0;
const SMALL_BUTTON_TEXT_SIZE: u16 = 50;
const DESCRIPTION_FONT_SIZE: f32 = 34.0;

const DECREASE_RENDER_DISTANCE_DESCRIPTION: [&str; 2] =
    ["Lower view distance,", "improves performance"];
const INCREASE_RENDER_DISTANCE_DESCRIPTION: [&str; 2] =
    ["Increase view distance,", "lowers performance"];
const TOGGLE_SOUNDS_DESCRIPTION: [&str; 1] = ["Toggles game sounds"];
const TOGGLE_FULLSCREEN_DESCRIPTION: [&str; 1] = ["Toggles fullscreen mode"];
const TOGGLE_LIGHTS_DESCRIPTION: [&str; 2] = [
    "Enables dynamic lights placed by the player",
    "and dynamic shadows, lowers performance",
];

pub struct SettingsContext;

impl SettingsContext {
    /// returns true if the settings menu should be closed
    pub async fn draw(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &mut UserSettings,
    ) -> InterfaceScreen {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, &asset_manager.texture_manager);
        let x_start = (width - BUTTON_WIDTH) * 0.5;
        let y_start = height * 0.3;

        Self::draw_settings_title(width, height, &asset_manager.font);
        Self::handle_render_distance(asset_manager, user_settings, width, y_start);
        Self::handle_toggle_sound_button(asset_manager, user_settings, x_start, y_start);
        Self::handle_toggle_fullscreen_button(asset_manager, user_settings, x_start, y_start);
        Self::handle_toggle_dynamic_light(asset_manager, user_settings, x_start, y_start);
        draw_version_number(height, &asset_manager.font);

        let should_exit = draw_back_button(asset_manager, user_settings);
        next_frame().await;

        if should_exit {
            write_user_settings(user_settings);
            InterfaceScreen::TitleScreen(TitleScreenContext::new())
        } else {
            InterfaceScreen::Settings(SettingsContext)
        }
    }

    fn draw_settings_title(width: f32, height: f32, font: &Font) {
        let settings_text = "Settings";
        let text_width = get_text_width(settings_text, SETTINGS_TEXT_WIDTH, font);
        let x = (width - text_width) * 0.5;
        let y = height * 0.1;

        draw_game_text(settings_text, x, y, SETTINGS_TEXT_WIDTH, TEXT_COLOR, font);
    }

    fn handle_render_distance(
        asset_manager: &AssetManager,
        user_settings: &mut UserSettings,
        width: f32,
        y_start: f32,
    ) {
        let x = (width - BUTTON_HEIGHT_OFFSET - RENDER_DISTANCE_TEXT_WIDTH) * 0.5;
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();
        Self::draw_description(
            width,
            height,
            &DECREASE_RENDER_DISTANCE_DESCRIPTION,
            is_point_in_rect(x, y_start, BUTTON_HEIGHT, BUTTON_HEIGHT, mouse_x, mouse_y),
            &asset_manager.font,
        );
        Self::draw_description(
            width,
            height,
            &INCREASE_RENDER_DISTANCE_DESCRIPTION,
            is_point_in_rect(
                x + RENDER_DISTANCE_TEXT_WIDTH,
                y_start,
                BUTTON_HEIGHT,
                BUTTON_HEIGHT,
                mouse_x,
                mouse_y,
            ),
            &asset_manager.font,
        );
        let decrease = draw_button(
            Rect {
                x,
                y: y_start,
                w: BUTTON_HEIGHT,
                h: BUTTON_HEIGHT,
            },
            "-",
            SMALL_BUTTON_TEXT_SIZE,
            asset_manager,
            user_settings,
        );

        let text = format!("View distance: {}", user_settings.get_render_distance());
        draw_game_text(
            &text,
            x + BUTTON_HEIGHT_OFFSET,
            y_start + BUTTON_TEXT_SIZE,
            BUTTON_TEXT_SIZE,
            TEXT_COLOR,
            &asset_manager.font,
        );

        let increase = draw_button(
            Rect {
                x: x + RENDER_DISTANCE_TEXT_WIDTH,
                y: y_start,
                w: BUTTON_HEIGHT,
                h: BUTTON_HEIGHT,
            },
            "+",
            SMALL_BUTTON_TEXT_SIZE,
            asset_manager,
            user_settings,
        );

        if increase {
            user_settings.increase_render_distance();
        } else if decrease {
            user_settings.decrease_render_distance();
        }
    }

    /// returns true if pressed
    fn handle_toggle_sound_button(
        asset_manager: &AssetManager,
        user_settings: &mut UserSettings,
        x_start: f32,
        y_start: f32,
    ) {
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();
        let y = y_start + BUTTON_HEIGHT_OFFSET;
        Self::draw_description(
            width,
            height,
            &TOGGLE_SOUNDS_DESCRIPTION,
            is_point_in_rect(x_start, y, BUTTON_WIDTH, BUTTON_HEIGHT, mouse_x, mouse_y),
            &asset_manager.font,
        );

        let should_toggle = draw_button(
            Rect {
                x: x_start,
                y,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            if user_settings.has_sound {
                "Sound:ON"
            } else {
                "Sound:OFF"
            },
            BUTTON_TEXT_SIZE as u16,
            asset_manager,
            user_settings,
        );

        if should_toggle {
            user_settings.has_sound = !user_settings.has_sound;
        }
    }

    fn handle_toggle_fullscreen_button(
        asset_manager: &AssetManager,
        user_settings: &mut UserSettings,
        x_start: f32,
        y_start: f32,
    ) {
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();
        let y = y_start + BUTTON_HEIGHT_OFFSET * 2.0;
        Self::draw_description(
            width,
            height,
            &TOGGLE_FULLSCREEN_DESCRIPTION,
            is_point_in_rect(x_start, y, BUTTON_WIDTH, BUTTON_HEIGHT, mouse_x, mouse_y),
            &asset_manager.font,
        );

        let should_change = draw_button(
            Rect {
                x: x_start,
                y,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            if user_settings.is_fullscreen {
                "Go windowed"
            } else {
                "Go fullscreen"
            },
            BUTTON_TEXT_SIZE as u16,
            asset_manager,
            user_settings,
        );
        if should_change {
            user_settings.is_fullscreen = !user_settings.is_fullscreen;
            set_fullscreen(user_settings.is_fullscreen);
        }
    }

    fn handle_toggle_dynamic_light(
        asset_manager: &AssetManager,
        user_settings: &mut UserSettings,
        x_start: f32,
        y_start: f32,
    ) {
        let (width, height) = screen_size();
        let (mouse_x, mouse_y) = mouse_position();
        let y = y_start + BUTTON_HEIGHT_OFFSET * 3.0;
        Self::draw_description(
            width,
            height,
            &TOGGLE_LIGHTS_DESCRIPTION,
            is_point_in_rect(x_start, y, BUTTON_WIDTH, BUTTON_HEIGHT, mouse_x, mouse_y),
            &asset_manager.font,
        );

        let should_change = draw_button(
            Rect {
                x: x_start,
                y,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            match user_settings.shadow_type {
                ShadowType::Soft => "Dynamic lights, soft shadows",
                ShadowType::Hard => "Dynamic lights, hard shadows",
                ShadowType::None => "Static lights and shadows",
            },
            BUTTON_TEXT_SIZE as u16,
            asset_manager,
            user_settings,
        );
        if should_change {
            Self::change_lighting_type(user_settings);
        }
    }

    fn change_lighting_type(user_settings: &mut UserSettings) {
        match user_settings.shadow_type {
            ShadowType::None => user_settings.shadow_type = ShadowType::Soft,
            ShadowType::Soft => user_settings.shadow_type = ShadowType::Hard,
            ShadowType::Hard => user_settings.shadow_type = ShadowType::None,
        }
    }

    fn draw_description(
        width: f32,
        height: f32,
        descriptions: &[&str],
        should_draw: bool,
        font: &Font,
    ) {
        if !should_draw {
            return;
        }

        draw_centered_multiline_text(
            descriptions,
            height * 0.92,
            width,
            DESCRIPTION_FONT_SIZE,
            TEXT_COLOR,
            font,
        );
    }
}
