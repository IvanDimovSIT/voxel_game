use macroquad::{
    camera::set_default_camera,
    color::{BLACK, WHITE},
    input::clear_input_queue,
    miniquad::window::screen_size,
    text::draw_text,
    window::next_frame,
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{background::draw_background, button::draw_button, util::get_text_width},
    model::user_settings::UserSettings,
    service::sound_manager::SoundManager,
};

const TITLE_TEXT: &str = "Voxel World";
const TITLE_SIZE: f32 = 120.0;
const BUTTON_TEXT_SIZE: u16 = 40;
const BUTTON_WIDTH: f32 = 250.0;
const BUTTON_HEIGHT: f32 = 100.0;

pub struct TitleScreenContext {
    should_exit: bool,
    should_play: bool,
}
impl TitleScreenContext {
    pub fn new() -> Self {
        clear_input_queue();
        Self {
            should_exit: false,
            should_play: false,
        }
    }

    fn draw_title(width: f32, height: f32) {
        let x = (width - get_text_width(TITLE_TEXT, TITLE_SIZE)) * 0.5;
        let y = height * 0.15;
        draw_text("Voxel World", x + 2.0, y + 2.0, TITLE_SIZE, BLACK);
        draw_text("Voxel World", x, y, TITLE_SIZE, WHITE);
    }

    fn draw_play_button(
        &self,
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        let x = (width - BUTTON_WIDTH) * 0.5;
        let y = height * 0.4;
        draw_button(
            x,
            y,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            "   Play",
            BUTTON_TEXT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    fn draw_exit_button(
        &self,
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        let x = (width - BUTTON_WIDTH) * 0.5;
        let y = height * 0.45 + BUTTON_HEIGHT;
        draw_button(
            x,
            y,
            BUTTON_WIDTH,
            BUTTON_HEIGHT,
            "   Exit",
            BUTTON_TEXT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    pub async fn draw(
        &mut self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, texture_manager);
        Self::draw_title(width, height);
        self.should_play = self.draw_play_button(width, height, sound_manager, user_settings);
        self.should_exit = self.draw_exit_button(width, height, sound_manager, user_settings);
        next_frame().await
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn should_play(&self) -> bool {
        self.should_play
    }
}
