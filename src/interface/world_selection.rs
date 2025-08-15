use std::rc::Rc;

use macroquad::{
    camera::set_default_camera,
    input::clear_input_queue,
    math::{Vec2, vec2},
    miniquad::window::screen_size,
    prelude::info,
    text::draw_text,
    window::next_frame,
};

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        background::draw_background, interface_context::InterfaceScreen, style::TEXT_COLOR,
        title_screen::TitleScreenContext, util::draw_centered_multiline_text,
    },
    model::user_settings::UserSettings,
    service::{
        persistence::{
            world_list_persistence::{read_world_list, write_world_list},
            world_persistence,
        },
        sound_manager::SoundManager,
    },
    voxel_engine::VoxelEngine,
};

use super::{
    button::draw_button, list_input::ListInput, text_input::TextInput, util::get_text_width,
};

const LABEL_FONT_SIZE: f32 = 40.0;
const TEXT_INPUT_SIZE: Vec2 = vec2(350.0, 50.0);
const TEXT_INPUT_FONT_SIZE: u16 = 36;
const PLAY_BUTTON_SIZE: Vec2 = vec2(220.0, 50.0);
const PLAY_BUTTON_FONT_SIZE: u16 = 40;
const BACK_BUTTON_SIZE: f32 = 60.0;
const BACK_BUTTON_FONT_SIZE: u16 = 45;
const NOTIFICATION_TEXT_SIZE: f32 = 35.0;
const DELETE_BUTTON_SIZE: Vec2 = vec2(220.0, 50.0);
const DELETE_BUTTON_FONT_SIZE: u16 = 35;
const WORLD_LIST_WIDTH: f32 = 450.0;
const WORLD_LIST_FONT_SIZE: f32 = 25.0;
const WORLD_LIST_ROWS: usize = 5;
const MIN_WORLD_NAME_LENGTH: usize = 3;

pub struct WorldSelectionContext {
    world_name_input: TextInput,
    error: String,
    should_enter: bool,
    world_list: ListInput,
}
impl WorldSelectionContext {
    pub fn new() -> Self {
        clear_input_queue();
        Self {
            world_name_input: TextInput::new(20),
            error: "".to_owned(),
            should_enter: false,
            world_list: ListInput::new(read_world_list(), WORLD_LIST_ROWS),
        }
    }

    pub fn enter_game(
        &self,
        texture_manager: Rc<TextureManager>,
        sound_manager: Rc<SoundManager>,
        user_settings: &UserSettings,
    ) -> Option<Box<VoxelEngine>> {
        if self.should_enter {
            self.store_world_names(true);
            let voxel_engine = Box::new(VoxelEngine::new(
                self.world_name_input.get_text(),
                texture_manager,
                sound_manager,
                user_settings.clone(),
            ));
            Some(voxel_engine)
        } else {
            None
        }
    }

    pub async fn draw(
        &mut self,
        texture_manager: &TextureManager,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> Option<InterfaceScreen> {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, texture_manager);

        Self::draw_input_label(width, height);
        self.handle_world_name_input(width, height);
        self.handle_world_list(width, height);
        self.handle_play_button(sound_manager, user_settings, width, height);

        let should_go_back = self.draw_back_button(sound_manager, user_settings);

        self.handle_delete_button(sound_manager, user_settings, width, height);

        self.draw_notification_text(width, height);

        next_frame().await;

        if should_go_back {
            Some(InterfaceScreen::TitleScreen(TitleScreenContext::new()))
        } else {
            None
        }
    }

    fn handle_play_button(
        &mut self,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
        width: f32,
        height: f32,
    ) {
        let play_button_pressed =
            self.draw_play_button(width, height, sound_manager, user_settings);
        if play_button_pressed {
            self.validate();
            self.should_enter = self.error.is_empty();
        }
    }

    fn handle_delete_button(
        &mut self,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
        width: f32,
        height: f32,
    ) {
        if let Some(selected_index) = self.world_list.get_selected_index() {
            let should_delete =
                self.draw_delete_button(width, height, sound_manager, user_settings);
            if should_delete {
                self.delete_world(
                    selected_index,
                    self.world_list.get_selected().unwrap_or_default(),
                );
                self.world_name_input.set_text("".to_owned());
            }
        }
    }

    fn draw_world_list_empty_text(&self, width: f32, height: f32) {
        let text = ["No worlds found", "Enter a name to create a new world"];
        let y = height * 0.7;

        draw_centered_multiline_text(&text, y, width, LABEL_FONT_SIZE, TEXT_COLOR);
    }

    fn handle_world_list(&mut self, width: f32, height: f32) {
        if self.world_list.len() == 0 {
            self.draw_world_list_empty_text(width, height);
            return;
        }

        let world_list_x = (width - WORLD_LIST_WIDTH) * 0.5;
        let world_list_y = height * 0.6;
        let possible_selection = self.world_list.draw(
            world_list_x,
            world_list_y,
            WORLD_LIST_WIDTH,
            WORLD_LIST_FONT_SIZE,
        );
        if let Some(selection) = possible_selection {
            self.world_name_input.set_text(selection);
        }
    }

    fn draw_notification_text(&mut self, width: f32, height: f32) {
        let text_to_notify = if self.should_enter {
            "Loading..."
        } else {
            &self.error
        };
        let notification_text_x =
            (width - get_text_width(text_to_notify, NOTIFICATION_TEXT_SIZE)) / 2.0;
        draw_text(
            text_to_notify,
            notification_text_x,
            height * 0.4,
            NOTIFICATION_TEXT_SIZE,
            TEXT_COLOR,
        );
    }

    /// returns true if pressed
    fn draw_delete_button(
        &mut self,
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        let delete_button_x = (width - DELETE_BUTTON_SIZE.x) / 2.0;
        draw_button(
            delete_button_x,
            height * 0.9,
            DELETE_BUTTON_SIZE.x,
            DELETE_BUTTON_SIZE.y,
            "Delete world",
            DELETE_BUTTON_FONT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    /// returns true if pressed
    fn draw_play_button(
        &mut self,
        width: f32,
        height: f32,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        let button_x = (width - PLAY_BUTTON_SIZE.x) * 0.5;
        let button_y = height * 0.5;
        draw_button(
            button_x,
            button_y,
            PLAY_BUTTON_SIZE.x,
            PLAY_BUTTON_SIZE.y,
            "Enter world",
            PLAY_BUTTON_FONT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    fn draw_back_button(
        &mut self,
        sound_manager: &SoundManager,
        user_settings: &UserSettings,
    ) -> bool {
        draw_button(
            10.0,
            10.0,
            BACK_BUTTON_SIZE,
            BACK_BUTTON_SIZE,
            "<",
            BACK_BUTTON_FONT_SIZE,
            sound_manager,
            user_settings,
        )
    }

    fn handle_world_name_input(&mut self, width: f32, height: f32) {
        let world_selection_x = (width - TEXT_INPUT_SIZE.x) / 2.0;
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
    }

    fn draw_input_label(width: f32, height: f32) {
        let text = "Enter world name:";
        let x = (width - get_text_width(text, LABEL_FONT_SIZE)) * 0.5;
        let y = height * 0.15;
        draw_text(text, x, y, LABEL_FONT_SIZE, TEXT_COLOR);
    }

    fn validate(&mut self) {
        let is_whitespace = || {
            self.world_name_input
                .get_text()
                .chars()
                .all(|c| c.is_ascii_whitespace())
        };

        if self.world_name_input.get_text().len() < MIN_WORLD_NAME_LENGTH {
            self.error =
                format!("World name should be at least {MIN_WORLD_NAME_LENGTH} characters");
        } else if is_whitespace() {
            self.error = "World name cannot be blank".to_owned();
        } else {
            self.error = "".to_owned();
        }
    }

    fn store_world_names(&self, include_from_input: bool) {
        let mut world_names = self.world_list.get_all_values();
        let world_names_contain_input = || {
            world_names
                .iter()
                .any(|x| x == self.world_name_input.get_text())
        };
        if include_from_input && !world_names_contain_input() {
            world_names.push(self.world_name_input.get_text().to_owned());
        }
        info!("Saving world list: {:?}", &world_names);
        write_world_list(&world_names);
    }

    fn delete_world(&mut self, index: usize, selected_value: String) {
        world_persistence::delete_world(&selected_value);
        self.world_list.remove(index);
        self.store_world_names(false);
    }
}
