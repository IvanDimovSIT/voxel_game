use macroquad::{
    camera::set_default_camera,
    miniquad::window::{screen_size, set_fullscreen},
    text::draw_text,
    window::next_frame,
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        background::draw_background, button::draw_button, interface_context::InterfaceScreen,
        style::TEXT_COLOR, title_screen::TitleScreenContext, util::get_text_width,
    },
    model::user_settings::UserSettings,
    service::{
        persistence::user_settings_persistence::write_user_settings, sound_manager::SoundManager,
    },
};

const BACK_BUTTON_SIZE: f32 = 60.0;
const BACK_BUTTON_FONT_SIZE: u16 = 45;
const BUTTON_WIDTH: f32 = 280.0;
const BUTTON_HEIGHT: f32 = 70.0;
const BUTTON_HEIGHT_OFFSET: f32 = BUTTON_HEIGHT * 1.2;
const BUTTON_TEXT_SIZE: f32 = 30.0;
const RENDER_DISTANCE_TEXT_WIDTH: f32 = 320.0;
const SETTINGS_TEXT_WIDTH: f32 = 80.0;
const SMALL_BUTTON_TEXT_SIZE: u16 = 50;

pub struct SettingsContext;

impl SettingsContext {
    /// returns true if the settings menu should be closed
    pub async fn draw(
        &mut self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &mut UserSettings,
    ) -> InterfaceScreen {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, texture_manager);
        let x_start = (width - BUTTON_WIDTH) * 0.5;
        let y_start = height * 0.3;

        Self::draw_settings_title(width, height);
        Self::handle_render_distance(sound_manager, user_settings, width, y_start);
        Self::handle_toggle_sound_button(sound_manager, user_settings, x_start, y_start);
        Self::handle_toggle_fullscreen_button(sound_manager, user_settings, x_start, y_start);

        let should_exit = Self::draw_back_button(sound_manager, user_settings);
        next_frame().await;

        if should_exit {
            write_user_settings(user_settings);
            InterfaceScreen::TitleScreen(TitleScreenContext::new())
        } else {
            InterfaceScreen::Settings(SettingsContext)
        }
    }

    fn draw_settings_title(width: f32, height: f32) {
        let settings_text = "Settings";
        let text_width = get_text_width(settings_text, SETTINGS_TEXT_WIDTH);
        let x = (width - text_width) * 0.5;
        let y = height * 0.1;

        draw_text(settings_text, x, y, SETTINGS_TEXT_WIDTH, TEXT_COLOR);
    }

    fn handle_render_distance(
        sound_manager: &SoundManager,
        user_settings: &mut UserSettings,
        width: f32,
        y_start: f32,
    ) {
        let x = (width - BUTTON_HEIGHT_OFFSET - RENDER_DISTANCE_TEXT_WIDTH) * 0.5;
        let decrease = draw_button(
            x,
            y_start,
            BUTTON_HEIGHT,
            BUTTON_HEIGHT,
            "-",
            SMALL_BUTTON_TEXT_SIZE,
            sound_manager,
            user_settings,
        );

        let text = format!("View distance: {}", user_settings.get_render_distance());
        draw_text(
            &text,
            x + BUTTON_HEIGHT_OFFSET,
            y_start + BUTTON_TEXT_SIZE,
            BUTTON_TEXT_SIZE,
            TEXT_COLOR,
        );

        let increase = draw_button(
            x + RENDER_DISTANCE_TEXT_WIDTH,
            y_start,
            BUTTON_HEIGHT,
            BUTTON_HEIGHT,
            "+",
            SMALL_BUTTON_TEXT_SIZE,
            sound_manager,
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
        sound_manager: &SoundManager,
        user_settings: &mut UserSettings,
        x_start: f32,
        y_start: f32,
    ) {
        let should_toggle = draw_button(
            x_start,
            y_start + BUTTON_HEIGHT_OFFSET,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            if user_settings.has_sound {
                "Sound:ON"
            } else {
                "Sound:OFF"
            },
            BUTTON_TEXT_SIZE as u16,
            sound_manager,
            user_settings,
        );

        if should_toggle {
            user_settings.has_sound = !user_settings.has_sound;
        }
    }

    fn handle_toggle_fullscreen_button(
        sound_manager: &SoundManager,
        user_settings: &mut UserSettings,
        x_start: f32,
        y_start: f32,
    ) {
        let should_change = draw_button(
            x_start,
            y_start + BUTTON_HEIGHT_OFFSET * 2.0,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            if user_settings.is_fullscreen {
                "Go windowed"
            } else {
                "Go fullscreen"
            },
            BUTTON_TEXT_SIZE as u16,
            sound_manager,
            user_settings,
        );
        if should_change {
            user_settings.is_fullscreen = !user_settings.is_fullscreen;
            set_fullscreen(user_settings.is_fullscreen);
        }
    }

    fn draw_back_button(sound_manager: &SoundManager, user_settings: &UserSettings) -> bool {
        draw_button(
            10.0,
            10.0,
            BACK_BUTTON_SIZE,
            BACK_BUTTON_SIZE,
            "<",
            BACK_BUTTON_FONT_SIZE,
            sound_manager,
            user_settings,
        )
    }
}
