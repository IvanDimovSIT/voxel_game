use macroquad::{
    camera::set_default_camera, color::BLACK, input::clear_input_queue, math::Rect,
    miniquad::window::screen_size, text::Font, window::next_frame,
};

use crate::{
    interface::{
        background::draw_background,
        button::draw_button,
        interface_context::InterfaceScreen,
        settings_menu::SettingsContext,
        style::TEXT_COLOR,
        util::{draw_game_text, draw_version_number, get_text_width},
        world_selection::WorldSelectionContext,
    },
    model::user_settings::UserSettings,
    service::asset_manager::AssetManager,
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
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> InterfaceScreen {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, &asset_manager.texture_manager);
        Self::draw_title(width, height, &asset_manager.font);
        let should_play = Self::draw_play_button(width, height, asset_manager, user_settings);
        let should_enter_settings =
            Self::draw_settings_button(width, height, asset_manager, user_settings);
        self.should_exit = Self::draw_exit_button(width, height, asset_manager, user_settings);
        draw_version_number(height, &asset_manager.font);
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

    fn draw_title(width: f32, height: f32, font: &Font) {
        let x = (width - get_text_width(TITLE_TEXT, TITLE_SIZE, font)) * 0.5;
        let y = height * 0.15;
        draw_game_text("Voxel World", x + 2.0, y + 2.0, TITLE_SIZE, BLACK, font);
        draw_game_text("Voxel World", x, y, TITLE_SIZE, TEXT_COLOR, font);
    }

    fn draw_title_screen_button(
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        text: &str,
        order: u32,
    ) -> bool {
        let x = (width - BUTTON_WIDTH) * 0.5;
        let y = height * 0.35 + BUTTON_HEIGHT_OFFSET * order as f32;
        draw_button(
            Rect {
                x,
                y,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            text,
            BUTTON_TEXT_SIZE,
            asset_manager,
            user_settings,
        )
    }

    fn draw_play_button(
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(width, height, asset_manager, user_settings, "   Play", 0)
    }

    fn draw_settings_button(
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(
            width,
            height,
            asset_manager,
            user_settings,
            "   Settings",
            1,
        )
    }

    fn draw_exit_button(
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> bool {
        Self::draw_title_screen_button(width, height, asset_manager, user_settings, "   Exit", 2)
    }
}
