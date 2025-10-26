use std::collections::HashSet;

use bincode::{Decode, Encode};
use macroquad::{
    shapes::draw_rectangle,
    text::{Font, TextDimensions, TextParams, draw_text_ex, measure_text},
};

use crate::{interface::style::SHADOW_COLOR, service::asset_manager::AssetManager};

const DISPLAY_MESSAGE_DURATION: f32 = 5.0;
const FONT_COEF: f32 = 0.05;
const MESSAGE_X: f32 = 20.0;
const Y_COEF: f32 = 0.8;
const MARGIN: f32 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum TutorialMessage {
    /// on entering world
    Initial,
    /// on opening map
    Map,
    /// on destroying voxel - explain place
    Destroy,
    /// on placeing voxel - explain replace
    Replacing,
    /// on hotbar filling up - explain inventory
    Inventory,
}
impl TutorialMessage {
    fn create_message_to_display(self) -> CurrentMessage {
        let texts = match self {
            TutorialMessage::Initial => vec![
                "Welcome to the world!",
                "Look around with the mouse and move using W/S/A/D",
                "Break voxels by pressing or holding the left mouse button",
            ],
            TutorialMessage::Map => vec![
                "Use W/S/A/D to rotate around the map, 'M' to exit",
                "You can zoom with the scroll wheel",
            ],
            TutorialMessage::Destroy => vec![
                "You collected some voxels, scroll to select them",
                "Place them with the right mouse button",
                "You can craft items by pressing 'C'",
            ],
            TutorialMessage::Replacing => {
                vec!["You can replace voxels using the middle mouse button"]
            }
            TutorialMessage::Inventory => vec!["Press 'E' to enter the inventory menu"],
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

    pub fn create_dto(&self) -> TutorialMessagesDTO {
        TutorialMessagesDTO {
            seen_messages: self.seen_messages.clone(),
        }
    }

    /// shows the message if it wasn't shown before
    pub fn show(&mut self, tutorial_message: TutorialMessage) {
        if self.current_message.is_some() {
            return;
        }

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

    pub fn draw(&self, height: f32, asset_manager: &AssetManager) {
        if let Some(message) = &self.current_message {
            debug_assert!(!message.texts.is_empty());
            debug_assert!(message.delta > 0.0);

            let text = message.texts[0];
            let font_size = (height * FONT_COEF) as u16;
            let font = Some(&asset_manager.font);
            let y = height * Y_COEF;

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
impl From<TutorialMessagesDTO> for TutorialMessages {
    fn from(value: TutorialMessagesDTO) -> Self {
        Self {
            seen_messages: value.seen_messages,
            current_message: None,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct TutorialMessagesDTO {
    seen_messages: HashSet<TutorialMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_should_add() {
        let mut tutorial = TutorialMessages::new();
        tutorial.show(TutorialMessage::Initial);

        assert!(tutorial.seen_messages.contains(&TutorialMessage::Initial));
        assert!(tutorial.current_message.is_some());
    }

    #[test]
    fn test_show_should_not_add() {
        let mut tutorial = TutorialMessages::new();

        tutorial.show(TutorialMessage::Initial);
        assert!(tutorial.current_message.is_some());

        tutorial.current_message = None;
        let seen_before = tutorial.seen_messages.clone();

        tutorial.show(TutorialMessage::Initial);

        assert_eq!(tutorial.seen_messages, seen_before);
        assert!(tutorial.current_message.is_none());
    }

    #[test]
    fn test_show_should_not_override_existing_message() {
        let mut tutorial = TutorialMessages::new();

        tutorial.show(TutorialMessage::Initial);
        assert!(tutorial.current_message.is_some());

        tutorial.show(TutorialMessage::Map);

        assert!(tutorial.seen_messages.contains(&TutorialMessage::Initial));
        assert!(!tutorial.seen_messages.contains(&TutorialMessage::Map));
        assert!(tutorial.current_message.is_some());
    }

    #[test]
    fn test_update_removes_messages() {
        let mut tutorial = TutorialMessages::new();
        tutorial.show(TutorialMessage::Initial);

        assert!(tutorial.current_message.is_some());
        let initial_text_count = tutorial.current_message.as_ref().unwrap().texts.len();

        tutorial.update(DISPLAY_MESSAGE_DURATION);
        assert_eq!(
            tutorial.current_message.as_ref().unwrap().texts.len(),
            initial_text_count - 1
        );

        while tutorial.current_message.is_some() {
            tutorial.update(DISPLAY_MESSAGE_DURATION);
        }

        assert!(tutorial.current_message.is_none());
    }

    #[test]
    fn test_create_and_convert_dto() {
        let mut tutorial = TutorialMessages::new();
        tutorial.show(TutorialMessage::Initial);
        tutorial.current_message = None;

        let dto = tutorial.create_dto();
        assert!(dto.seen_messages.contains(&TutorialMessage::Initial));

        let converted: TutorialMessages = dto.into();
        assert!(converted.seen_messages.contains(&TutorialMessage::Initial));
        assert!(converted.current_message.is_none());
    }
}
