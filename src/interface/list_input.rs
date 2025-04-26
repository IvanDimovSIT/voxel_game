use macroquad::{
    color::BLACK,
    input::{is_mouse_button_pressed, mouse_position},
    miniquad::window::screen_size,
    shapes::draw_rectangle,
    text::draw_text,
};

use crate::service::input::{ScrollDirection, get_scroll_direction};

use super::{
    style::{BUTTON_COLOR, BUTTON_HOVER_COLOR, SELECTED_COLOR},
    util::{draw_rect_with_shadow, is_point_in_rect},
};

#[derive(Debug)]
pub struct ListInput {
    values: Vec<String>,
    selected: Option<usize>,
    rows: usize,
    current_page: usize,
}
impl ListInput {
    pub fn new(values: Vec<String>, rows: usize) -> Self {
        Self {
            values,
            selected: None,
            rows,
            current_page: 0,
        }
    }

    pub fn get_selected(&self) -> Option<String> {
        self.selected.map(|index| self.values[index].clone())
    }

    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.values.len() {
            return;
        }

        self.values.remove(index);
        if let Some(selected) = self.selected {
            if selected > index {
                self.selected = Some(selected - 1);
            }
            if self.values.is_empty() {
                self.selected = None;
            }
        }
    }

    /// returns the newly selected value
    pub fn draw(&mut self, x: f32, y: f32, w: f32, font_size: f32) -> Option<String> {
        let mut newly_selected = None;
        self.scroll();
        let start_index = self.current_page * self.rows;
        let end_index = ((self.current_page + 1) * self.rows).min(self.values.len());
        let values_on_page = &self.values[start_index..end_index];
        let y_offset = font_size * 1.25;
        if let Some(selected) = self.selected {
            self.selected = Some(selected.min(self.values.len()));
        };
        draw_rect_with_shadow(x, y, w, y_offset * self.rows as f32, BUTTON_COLOR);

        let (mouse_x, mouse_y) = mouse_position();
        for (index, value) in values_on_page.iter().enumerate() {
            let row_y = y + index as f32 * y_offset;
            let is_selected = self.selected.is_some()
                && self.selected.unwrap() == index + self.current_page * self.rows;
            let is_mouseover = is_point_in_rect(x, row_y, w, y_offset, mouse_x, mouse_y);
            let (bg_color, text_color) = if !is_selected && is_mouseover {
                (BUTTON_HOVER_COLOR, BUTTON_COLOR)
            } else {
                (SELECTED_COLOR, BLACK)
            };

            if is_selected || is_mouseover {
                draw_rectangle(x, y + index as f32 * y_offset, w, y_offset, bg_color);
            }
            draw_text(
                &value,
                x + 2.0,
                y + (index as f32 + 1.0) * y_offset - y_offset * 0.1,
                font_size,
                text_color,
            );
            if is_mouseover && is_mouse_button_pressed(macroquad::input::MouseButton::Left) {
                self.selected = Some(index + self.current_page * self.rows);
                newly_selected = self.get_selected();
            }
        }

        newly_selected
    }

    pub fn get_all_values(&self) -> Vec<String> {
        self.values.clone()
    }

    fn scroll(&mut self) {
        let total_pages = self.values.len() / self.rows;
        self.current_page = self.current_page.clamp(0, total_pages);

        match get_scroll_direction() {
            ScrollDirection::Up => {
                if self.current_page > 0 {
                    self.current_page = (self.current_page - 1).min(total_pages);
                }
            }
            ScrollDirection::Down => {
                self.current_page = (self.current_page + 1).min(total_pages);
            }
            ScrollDirection::None => {}
        }
    }
}
