use macroquad::{
    camera::set_default_camera,
    input::clear_input_queue,
    math::{Rect, Vec2, vec2},
    miniquad::window::screen_size,
    text::Font,
    window::next_frame,
};

use crate::{
    interface::{
        background::draw_background,
        button::draw_button,
        help_menu::HelpMenuContext,
        interface_context::InterfaceScreen,
        settings_menu::SettingsContext,
        style::TEXT_COLOR,
        text::{draw_text_with_shadow, draw_version_number, get_text_width},
        world_selection::WorldSelectionContext,
    },
    model::user_settings::UserSettings,
    service::asset_manager::AssetManager,
};

const TITLE_TEXT: &str = "Voxel World";
const TITLE_SIZE: f32 = 120.0;
const BUTTON_TEXT_SIZE: u16 = 40;
const BUTTON_WIDTH: f32 = 300.0;
const BUTTON_HEIGHT: f32 = 70.0;
const BUTTON_HEIGHT_OFFSET: f32 = BUTTON_HEIGHT * 1.2;
const TITLE_LOCATION_Y: f32 = 0.15;
const TITLE_BUTTONS_START_LOCATION_Y: f32 = 0.35;
const TITLE_SHADOW_OFFSET: Vec2 = vec2(3.0, 3.0);
const PLAY_BUTTON_ORDER: u32 = 0;
const SETTINGS_BUTTON_ORDER: u32 = 1;
const HELP_BUTTON_ORDER: u32 = 2;
const EXIT_BUTTON_ORDER: u32 = 3;

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
        let should_enter_help_menu =
            Self::draw_help_button(width, height, asset_manager, user_settings);
        self.should_exit = Self::draw_exit_button(width, height, asset_manager, user_settings);
        draw_version_number(height, &asset_manager.font);
        next_frame().await;

        if should_enter_settings {
            InterfaceScreen::Settings(SettingsContext)
        } else if should_play {
            InterfaceScreen::WorldSelection(WorldSelectionContext::new())
        } else if should_enter_help_menu {
            InterfaceScreen::Help(HelpMenuContext::new(asset_manager))
        } else {
            InterfaceScreen::TitleScreen(self.clone())
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn draw_title(width: f32, height: f32, font: &Font) {
        let x = (width - get_text_width(TITLE_TEXT, TITLE_SIZE, font)) / 2.0;
        let y = height * TITLE_LOCATION_Y;

        draw_text_with_shadow(
            "Voxel World",
            vec2(x, y),
            TITLE_SHADOW_OFFSET,
            TITLE_SIZE,
            TEXT_COLOR,
            font,
        );
    }

    fn draw_title_screen_button(
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        text: &str,
        order: u32,
    ) -> bool {
        let x = (width - BUTTON_WIDTH) / 2.0;
        let y = height * TITLE_BUTTONS_START_LOCATION_Y + BUTTON_HEIGHT_OFFSET * order as f32;
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
        Self::draw_title_screen_button(
            width,
            height,
            asset_manager,
            user_settings,
            "   Play",
            PLAY_BUTTON_ORDER,
        )
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
            SETTINGS_BUTTON_ORDER,
        )
    }

    fn draw_help_button(
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
            "   Help",
            HELP_BUTTON_ORDER,
        )
    }

    fn draw_exit_button(
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
            "   Exit",
            EXIT_BUTTON_ORDER,
        )
    }
}
