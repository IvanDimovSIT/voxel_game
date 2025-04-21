use std::rc::Rc;

use macroquad::{
    camera::set_default_camera,
    color::WHITE,
    input::clear_input_queue,
    math::{Vec2, vec2},
    miniquad::window::screen_size,
    text::draw_text,
    window::{clear_background, next_frame},
};

use crate::{
    graphics::texture_manager::TextureManager, service::sound_manager::SoundManager,
    voxel_engine::VoxelEngine,
};

use super::{
    button::draw_button, style::BACKGROUND_COLOR, text_input::TextInput, util::get_text_width,
};

const LABEL_FONT_SIZE: f32 = 40.0;
const TEXT_INPUT_SIZE: Vec2 = vec2(350.0, 50.0);
const TEXT_INPUT_FONT_SIZE: u16 = 35;
const PLAY_BUTTON_SIZE: Vec2 = vec2(220.0, 50.0);
const PLAY_BUTTON_FONT_SIZE: u16 = 40;
const NOTIFICATION_TEXT_SIZE: f32 = 35.0;

pub struct InterfaceContext {
    world_name_input: TextInput,
    error: String,
    should_enter: bool,
}
impl InterfaceContext {
    pub fn new() -> Self {
        clear_input_queue();
        Self {
            world_name_input: TextInput::new(20),
            error: "".to_owned(),
            should_enter: false,
        }
    }

    pub fn enter_game(
        &self,
        texture_manager: Rc<TextureManager>,
        sound_manager: Rc<SoundManager>,
    ) -> Option<Box<VoxelEngine>> {
        if self.should_enter {
            let voxel_engine = Box::new(VoxelEngine::new(
                self.world_name_input.get_text(),
                texture_manager,
                sound_manager,
            ));
            Some(voxel_engine)
        } else {
            None
        }
    }

    pub async fn draw(&mut self) {
        set_default_camera();
        clear_background(BACKGROUND_COLOR);
        let (width, height) = screen_size();
        Self::draw_input_label(width, height);
        let world_selection_x = (width - TEXT_INPUT_SIZE.x) * 0.5;
        let world_selection_y = height * 0.2;
        let _set_selected = self.world_name_input.input_selection(
            world_selection_x,
            world_selection_y,
            TEXT_INPUT_SIZE.x,
            TEXT_INPUT_SIZE.y,
        );
        self.world_name_input.input_text();

        self.world_name_input.draw(
            world_selection_x,
            world_selection_y,
            TEXT_INPUT_SIZE.x,
            TEXT_INPUT_SIZE.y,
            TEXT_INPUT_FONT_SIZE,
        );
        if !self.error.is_empty() {
            self.validate();
        }

        let button_x = (width - PLAY_BUTTON_SIZE.x) * 0.5;
        let button_y = height * 0.5;
        let play_button_pressed = draw_button(
            button_x,
            button_y,
            PLAY_BUTTON_SIZE.x,
            PLAY_BUTTON_SIZE.y,
            "Enter world",
            PLAY_BUTTON_FONT_SIZE,
        );

        if play_button_pressed {
            self.validate();
            self.should_enter = self.error.is_empty();
        }

        let text_to_notify = if self.should_enter {
            "Loading..."
        } else {
            &self.error
        };
        let notification_text_x =
            (width - get_text_width(text_to_notify, NOTIFICATION_TEXT_SIZE)) * 0.5;
        draw_text(
            text_to_notify,
            notification_text_x,
            height * 0.4,
            NOTIFICATION_TEXT_SIZE,
            WHITE,
        );

        next_frame().await;
    }

    fn draw_input_label(width: f32, height: f32) {
        let text = "Enter world name:";
        let x = (width - get_text_width(text, LABEL_FONT_SIZE)) * 0.5;
        let y = height * 0.15;
        draw_text(text, x, y, LABEL_FONT_SIZE, WHITE);
    }

    fn validate(&mut self) {
        if self.world_name_input.get_text().len() < 3 {
            self.error = "World name should be at least 3 characters".to_owned();
        } else {
            self.error = "".to_owned();
        }
    }
}
