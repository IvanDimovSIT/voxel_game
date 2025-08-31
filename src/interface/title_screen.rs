use macroquad::{
    camera::set_default_camera, color::BLACK, input::clear_input_queue,
    miniquad::window::screen_size, text::draw_text, window::next_frame,
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        background::draw_background,
        button::draw_button,
        interface_context::InterfaceScreen,
        settings_menu::SettingsContext,
        style::TEXT_COLOR,
        util::{draw_version_number, get_text_width},
        world_selection::WorldSelectionContext,
    },
    model::user_settings::UserSettings,
    service::sound_manager::SoundManager,
};

const TITLE_TEXT: &str = "Voxel World";
const TITLE_SIZE: f32 = 120.0;
const BUTTON_TEXT_SIZE: u16 = 40;
const BUTTON_WIDTH: f32 = 280.0;
const BUTTON_HEIGHT: f32 = 80.0;
const BUTTON_HEIGHT_OFFSET: f32 = BUTTON_HEIGHT * 1.2;

#[derive(Clone)]
pub struct TitleScreenContext {
    should_exit: bool,
}
impl TitleScreenContext {
    pub fn new() -> Self {
        clear_input_queue();
        Self { should_exit: false }
    }

    pub async fn draw(
        &mut self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> InterfaceScreen {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, texture_manager);
        Self::draw_title(width, height);
        let should_play = Self::draw_play_button(width, height, sound_manager, user_settings);
        let should_enter_settings =
            Self::draw_settings_button(width, height, sound_manager, user_settings);
        self.should_exit = Self::draw_exit_button(width, height, sound_manager, user_settings);
        draw_version_number(height);
        next_frame().await;

        if should_enter_settings {
            InterfaceScreen::Settings(SettingsContext)
        } else if should_play {
            InterfaceScreen::WorldSelection(WorldSelectionContext::new())
        } else {
            InterfaceScreen::TitleScreen(self.clone())
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn draw_title(width: f32, height: f32) {
        let x = (width - get_text_width(TITLE_TEXT, TITLE_SIZE)) * 0.5;
        let y = height * 0.15;
        draw_text("Voxel World", x + 2.0, y + 2.0, TITLE_SIZE, BLACK);
        draw_text("Voxel World", x, y, TITLE_SIZE, TEXT_COLOR);
    }

    fn draw_title_screen_button(
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
        text: &str,
        order: u32,
    ) -> bool {
        let x = (width - BUTTON_WIDTH) * 0.5;
        let y = height * 0.35 + BUTTON_HEIGHT_OFFSET * order as f32;
        draw_button(
            x,
            y,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            text,
            BUTTON_TEXT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    fn draw_play_button(
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(width, height, sound_manager, user_settings, "   Play", 0)
    }

    fn draw_settings_button(
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(
            width,
            height,
            sound_manager,
            user_settings,
            "   Settings",
            1,
        )
    }

    fn draw_exit_button(
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(width, height, sound_manager, user_settings, "   Exit", 2)
    }
}
