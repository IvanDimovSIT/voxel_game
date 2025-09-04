use std::borrow::Cow;

use macroquad::{
    color::BLACK,
    input::{
        clear_input_queue, get_char_pressed, is_key_released, is_mouse_button_released,
        mouse_position,
    },
    shapes::draw_rectangle_lines,
    text::Font,
};

use crate::interface::util::draw_game_text;

use super::{
    style::*,
    util::{draw_rect_with_shadow, is_point_in_rect},
};

#[derive(Debug)]
pub struct TextInput {
    text: String,
    is_selected: bool,
    max_length: usize,
}
impl TextInput {
    pub fn new(max_length: usize) -> Self {
        Self {
            text: "".to_owned(),
            is_selected: false,
            max_length,
        }
    }

    /// selects the text input and returns if it has just been selected
    pub fn input_selection(&mut self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let (mouse_x, mouse_y) = mouse_position();
        let has_clicked = is_mouse_button_released(macroquad::input::MouseButton::Left);
        if !has_clicked {
            return false;
        }
        let is_selected = has_clicked && is_point_in_rect(x, y, w, h, mouse_x, mouse_y);
        if is_selected {
            clear_input_queue();
        }
        self.is_selected = is_selected;
        is_selected
    }

    pub fn input_text(&mut self) {
        if !self.is_selected {
            return;
        }
        if is_key_released(macroquad::input::KeyCode::Backspace) {
            self.text.pop();
            return;
        }

        let mut chars = vec![];
        loop {
            let char = get_char_pressed();
            if let Some(char) = char {
                chars.push(char);
            } else {
                break;
            }
        }

        let characters: String = chars
            .into_iter()
            .filter(|c| Self::is_character_allowed(*c))
            .collect();

        self.text += &characters;
        self.text.truncate(self.max_length);
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn draw(&self, x: f32, y: f32, w: f32, h: f32, text_size: u16, font: &Font) {
        let (mouse_x, mouse_y) = mouse_position();
        let is_hovered = is_point_in_rect(x, y, w, h, mouse_x, mouse_y);
        let text_input_color = if is_hovered {
            BUTTON_HOVER_COLOR
        } else {
            BUTTON_COLOR
        };

        let (border_size, border_color, text_to_draw) = if self.is_selected {
            (5.0, SELECTED_COLOR, Cow::Owned(format!("{}|", &self.text)))
        } else {
            (2.0, BLACK, Cow::Borrowed(&self.text))
        };

        draw_rect_with_shadow(x, y, w, h, text_input_color);
        draw_rectangle_lines(x, y, w, h, border_size, border_color);
        draw_game_text(
            &text_to_draw,
            x + MARGIN,
            y + h * 0.5 + text_size as f32 * 0.5,
            text_size,
            BLACK,
            font,
        );
    }

    pub fn set_text(&mut self, new_text: String) {
        if new_text.chars().all(Self::is_character_allowed) {
            self.text = new_text;
            self.text.truncate(self.max_length);
        }
    }

    fn is_character_allowed(character: char) -> bool {
        character.is_alphanumeric() || character == '_' || character == ' '
    }
}
