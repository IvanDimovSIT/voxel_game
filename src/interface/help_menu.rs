use macroquad::{
    color::WHITE,
    math::vec2,
    miniquad::window::screen_size,
    text::Font,
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
    window::next_frame,
};

use crate::{
    interface::{
        background::draw_background,
        button::draw_back_button,
        interface_context::InterfaceScreen,
        style::{MENU_TITLE_FONT_SIZE, TEXT_COLOR},
        text::{
            draw_centered_multiline_text, draw_game_text, draw_multiline_left_text,
            draw_version_number, get_text_width,
        },
        title_screen::TitleScreenContext,
    },
    model::user_settings::UserSettings,
    service::asset_manager::AssetManager,
};

const FONT_SIZE_COEF: f32 = 0.055;
const TITLE_TEXT_Y_COEF: f32 = 0.1;
const HELP_TEXT_Y_COEF: f32 = 0.45;

const IMAGE_Y_COEF: f32 = 0.1;
const BASE_IMAGE_DESCRIPTION_SIZE: f32 = 40.0;
const IMAGE_SIZE_COEF: f32 = 0.002;
const IMAGE_BASE_WIDTH: f32 = 256.0;
const IMAGE_BASE_HEIGHT: f32 = 128.0;

pub struct HelpMenuContext {
    controls_image: Texture2D,
}
impl HelpMenuContext {
    pub fn new(asset_manager: &AssetManager) -> Self {
        Self {
            controls_image: asset_manager.texture_manager.get_controls_image(),
        }
    }

    /// returns the new screen if changed
    pub async fn draw(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> Option<InterfaceScreen> {
        let (width, height) = screen_size();
        draw_background(width, height, &asset_manager.texture_manager);
        draw_version_number(height, &asset_manager.font);
        self.draw_controls_image(width, height, asset_manager);
        Self::draw_help_title(width, height, &asset_manager.font);
        Self::draw_help_text(width, height, asset_manager);
        let should_go_back = draw_back_button(asset_manager, user_settings);

        next_frame().await;

        if should_go_back {
            Some(InterfaceScreen::TitleScreen(TitleScreenContext::new()))
        } else {
            None
        }
    }

    fn draw_controls_image(&self, width: f32, height: f32, asset_manager: &AssetManager) {
        let coef = height * IMAGE_SIZE_COEF;
        let image_width = IMAGE_BASE_WIDTH * coef;
        let image_height = IMAGE_BASE_HEIGHT * coef;
        let x = (width - image_width) / 2.0;
        let y = height * IMAGE_Y_COEF;

        draw_texture_ex(
            &self.controls_image,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(image_width, image_height)),
                ..Default::default()
            },
        );

        draw_centered_multiline_text(
            &["Move     Look"],
            y + image_height,
            width,
            BASE_IMAGE_DESCRIPTION_SIZE * coef,
            WHITE,
            &asset_manager.font,
        );
    }

    fn draw_help_text(width: f32, height: f32, asset_manager: &AssetManager) {
        let y = height * HELP_TEXT_Y_COEF;
        let font_size = height * FONT_SIZE_COEF;
        let help_text = [
            "Controls:",
            "W/S/A/D - Move",
            "Left mouse - Break voxels",
            "Right mouse - Place voxels",
            "Middle mouse button - Replace voxels",
            "Scroll/1-8 - Change selected voxel",
            "E - Inventory",
            "C - Crafting",
            "Escape - Game menu",
            "M - Toggles a map of the world",
        ];
        draw_multiline_left_text(
            &help_text,
            y,
            width,
            font_size,
            TEXT_COLOR,
            &asset_manager.font,
        );
    }

    fn draw_help_title(width: f32, height: f32, font: &Font) {
        let text = "Help";
        let text_width = get_text_width(text, MENU_TITLE_FONT_SIZE, font);
        let x = (width - text_width) / 2.0;
        let y = height * TITLE_TEXT_Y_COEF;

        draw_game_text(text, x, y, MENU_TITLE_FONT_SIZE, TEXT_COLOR, font);
    }
}
