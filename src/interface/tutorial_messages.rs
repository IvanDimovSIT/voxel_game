use std::collections::HashSet;

use macroquad::{
    shapes::draw_rectangle,
    text::{Font, TextDimensions, TextParams, draw_text_ex, measure_text},
};

use crate::{interface::style::SHADOW_COLOR, service::asset_manager::AssetManager};

const DISPLAY_MESSAGE_DURATION: f32 = 5.0;
const FONT_COEF: f32 = 0.05;
const MESSAGE_X: f32 = 20.0;
const Y_COEF: f32 = 0.6;
const MARGIN: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TutorialMessage {
    Initial,
    Map,
    Crafting,
}
impl TutorialMessage {
    fn create_message_to_display(self) -> CurrentMessage {
        let texts = match self {
            TutorialMessage::Initial => vec!["Welcome to the world!"],
            TutorialMessage::Map => vec![
                "Use W/S/A/D to rotate around the map, 'M' to exit",
                "You can zoom with the scroll wheel",
            ],
            TutorialMessage::Crafting => vec!["You can craft items by pressing 'C'"],
        };

        CurrentMessage::new(texts)
    }
}

struct CurrentMessage {
    texts: Vec<&'static str>,
    delta: f32,
}
impl CurrentMessage {
    fn new(texts: Vec<&'static str>) -> Self {
        Self {
            texts,
            delta: DISPLAY_MESSAGE_DURATION,
        }
    }
}

pub struct TutorialMessages {
    seen_messages: HashSet<TutorialMessage>,
    current_message: Option<CurrentMessage>,
}
impl TutorialMessages {
    pub fn new() -> Self {
        Self {
            seen_messages: HashSet::new(),
            current_message: None,
        }
    }

    /// shows the message if it wasn't shown before
    pub fn show(&mut self, tutorial_message: TutorialMessage) {
        let message_already_displayed = !self.seen_messages.insert(tutorial_message);
        if message_already_displayed {
            return;
        }
        self.current_message = Some(tutorial_message.create_message_to_display());
    }

    pub fn update(&mut self, delta: f32) {
        if let Some(current_message) = &mut self.current_message {
            current_message.delta -= delta;
            if current_message.delta > 0.0 {
                return;
            }

            current_message.texts.remove(0);
            if current_message.texts.is_empty() {
                self.current_message = None;
            } else {
                current_message.delta = DISPLAY_MESSAGE_DURATION;
            }
        }
    }

    pub fn draw(&self, width: f32, height: f32, asset_manager: &AssetManager) {
        if let Some(message) = &self.current_message {
            debug_assert!(!message.texts.is_empty());
            debug_assert!(message.delta > 0.0);

            let text = message.texts[0];
            let font_size = (height * FONT_COEF) as u16;
            let font = Some(&asset_manager.font);
            let y = width * Y_COEF;

            let text_dimensions = measure_text(text, font, font_size, 1.0);

            Self::draw_message_background(y, text_dimensions);
            Self::draw_message_text(text, y, font, font_size, text_dimensions);
        }
    }

    fn draw_message_text(
        text: &str,
        message_y: f32,
        font: Option<&Font>,
        font_size: u16,
        text_dimensions: TextDimensions,
    ) {
        let x = MESSAGE_X + MARGIN;
        let y = message_y + text_dimensions.offset_y + MARGIN;

        draw_text_ex(
            text,
            x,
            y,
            TextParams {
                font,
                font_size,
                ..Default::default()
            },
        );
    }

    fn draw_message_background(y: f32, text_dimensions: TextDimensions) {
        let text_width = text_dimensions.width;
        let text_height = text_dimensions.height;

        draw_rectangle(
            MESSAGE_X,
            y,
            text_width + MARGIN * 2.0,
            text_height + MARGIN * 2.0,
            SHADOW_COLOR,
        );
    }
}
