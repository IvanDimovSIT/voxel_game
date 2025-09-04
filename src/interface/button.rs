use macroquad::{
    color::BLACK,
    input::{is_mouse_button_released, mouse_position},
    math::Rect,
};

use crate::{
    interface::util::draw_game_text,
    model::user_settings::UserSettings,
    service::{asset_manager::AssetManager, sound_manager::SoundId},
};

use super::{
    style::*,
    util::{draw_rect_with_shadow, is_point_in_rect},
};

/// draws a button and returns if it is pressed
pub fn draw_button(
    rect: Rect,
    text: &str,
    text_size: u16,
    asset_manager: &AssetManager,
    user_settings: &UserSettings,
) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let is_hovered = is_point_in_rect(rect.x, rect.y, rect.w, rect.h, mouse_x, mouse_y);
    let button_color = if is_hovered {
        BUTTON_HOVER_COLOR
    } else {
        BUTTON_COLOR
    };

    draw_rect_with_shadow(rect.x, rect.y, rect.w, rect.h, button_color);
    draw_game_text(
        text,
        rect.x + MARGIN,
        rect.y + rect.h * 0.5 + text_size as f32 * 0.5,
        text_size,
        BLACK,
        &asset_manager.font,
    );

    let is_clicked = is_hovered && is_mouse_button_released(macroquad::input::MouseButton::Left);
    if is_clicked {
        asset_manager
            .sound_manager
            .play_sound(SoundId::Click, user_settings);
    }

    is_clicked
}
