use std::rc::Rc;

use macroquad::{
    camera::set_default_camera,
    input::clear_input_queue,
    math::{Rect, Vec2, vec2},
    miniquad::window::screen_size,
    prelude::info,
    text::Font,
    window::next_frame,
};

use crate::{
    interface::{
        background::draw_background,
        button::draw_back_button,
        interface_context::InterfaceScreen,
        style::TEXT_COLOR,
        text::{draw_centered_multiline_text, draw_game_text, draw_version_number},
        title_screen::TitleScreenContext,
    },
    model::user_settings::UserSettings,
    service::{
        asset_manager::AssetManager,
        persistence::{
            world_list_persistence::{read_world_list, write_world_list},
            world_persistence,
        },
    },
    voxel_engine::VoxelEngine,
};

use super::{
    button::draw_button, list_input::ListInput, text::get_text_width, text_input::TextInput,
};

const LABEL_FONT_SIZE: f32 = 40.0;
const TEXT_INPUT_SIZE: Vec2 = vec2(350.0, 50.0);
const TEXT_INPUT_FONT_SIZE: u16 = 36;
const PLAY_BUTTON_SIZE: Vec2 = vec2(220.0, 50.0);
const PLAY_BUTTON_FONT_SIZE: u16 = 40;
const PLAY_BUTTON_Y_COEF: f32 = 0.5;
const NOTIFICATION_TEXT_SIZE: f32 = 35.0;
const DELETE_BUTTON_SIZE: Vec2 = vec2(220.0, 50.0);
const DELETE_BUTTON_FONT_SIZE: u16 = 35;
const DELETE_BUTTON_Y_COEF: f32 = 0.9;
const WORLD_LIST_WIDTH: f32 = 450.0;
const WORLD_LIST_FONT_SIZE: f32 = 25.0;
const WORLD_LIST_ROWS: usize = 5;
const MIN_WORLD_NAME_LENGTH: usize = 3;
const WORLD_NAME_INPUT_Y_COEF: f32 = 0.2;

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
        asset_manager: Rc<AssetManager>,
        user_settings: &UserSettings,
    ) -> Option<Box<VoxelEngine>> {
        if self.should_enter {
            self.store_world_names(true);
            let voxel_engine = Box::new(VoxelEngine::new(
                self.world_name_input.get_text(),
                asset_manager,
                user_settings.clone(),
            ));
            Some(voxel_engine)
        } else {
            None
        }
    }

    pub async fn draw(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> Option<InterfaceScreen> {
        set_default_camera();
        let (width, height) = screen_size();
        draw_background(width, height, &asset_manager.texture_manager);

        Self::draw_input_label(width, height, &asset_manager.font);
        self.handle_world_name_input(width, height, &asset_manager.font);
        self.handle_world_list(width, height, &asset_manager.font);
        self.handle_play_button(asset_manager, user_settings, width, height);

        let should_go_back = draw_back_button(asset_manager, user_settings);

        self.handle_delete_button(asset_manager, user_settings, width, height);

        self.draw_notification_text(width, height, &asset_manager.font);
        draw_version_number(height, &asset_manager.font);

        next_frame().await;

        if should_go_back {
            Some(InterfaceScreen::TitleScreen(TitleScreenContext::new()))
        } else {
            None
        }
    }

    fn handle_play_button(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        width: f32,
        height: f32,
    ) {
        let play_button_pressed =
            self.draw_play_button(width, height, asset_manager, user_settings);
        if play_button_pressed {
            self.validate();
            self.should_enter = self.error.is_empty();
        }
    }

    fn handle_delete_button(
        &mut self,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        width: f32,
        height: f32,
    ) {
        if let Some(selected_index) = self.world_list.get_selected_index() {
            let should_delete =
                self.draw_delete_button(width, height, asset_manager, user_settings);
            if should_delete {
                self.delete_world(
                    selected_index,
                    self.world_list.get_selected().unwrap_or_default(),
                );
                self.world_name_input.set_text("".to_owned());
            }
        }
    }

    fn draw_world_list_empty_text(&self, width: f32, height: f32, font: &Font) {
        let text = ["No worlds found", "Enter a name to create a new world"];
        let y = height * 0.7;

        draw_centered_multiline_text(&text, y, width, LABEL_FONT_SIZE, TEXT_COLOR, font);
    }

    fn handle_world_list(&mut self, width: f32, height: f32, font: &Font) {
        if self.world_list.len() == 0 {
            self.draw_world_list_empty_text(width, height, font);
            return;
        }

        let world_list_x = (width - WORLD_LIST_WIDTH) / 2.0;
        let world_list_y = height * 0.6;
        let possible_selection = self.world_list.draw(
            world_list_x,
            world_list_y,
            WORLD_LIST_WIDTH,
            WORLD_LIST_FONT_SIZE,
            font,
        );
        if let Some(selection) = possible_selection {
            self.world_name_input.set_text(selection);
        }
    }

    fn draw_notification_text(&mut self, width: f32, height: f32, font: &Font) {
        let text_to_notify = if self.should_enter {
            "Loading..."
        } else {
            &self.error
        };
        let notification_text_x =
            (width - get_text_width(text_to_notify, NOTIFICATION_TEXT_SIZE as u16, font)) / 2.0;
        draw_game_text(
            text_to_notify,
            notification_text_x,
            height * 0.4,
            NOTIFICATION_TEXT_SIZE,
            TEXT_COLOR,
            font,
        );
    }

    /// returns true if pressed
    fn draw_delete_button(
        &mut self,
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> bool {
        let delete_button_x = (width - DELETE_BUTTON_SIZE.x) / 2.0;
        draw_button(
            Rect {
                x: delete_button_x,
                y: height * DELETE_BUTTON_Y_COEF,
                w: DELETE_BUTTON_SIZE.x,
                h: DELETE_BUTTON_SIZE.y,
            },
            "Delete world",
            DELETE_BUTTON_FONT_SIZE,
            asset_manager,
            user_settings,
        )
    }

    /// returns true if pressed
    fn draw_play_button(
        &mut self,
        width: f32,
        height: f32,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
    ) -> bool {
        let button_x = (width - PLAY_BUTTON_SIZE.x) / 2.0;
        let button_y = height * PLAY_BUTTON_Y_COEF;
        draw_button(
            Rect {
                x: button_x,
                y: button_y,
                w: PLAY_BUTTON_SIZE.x,
                h: PLAY_BUTTON_SIZE.y,
            },
            "Enter world",
            PLAY_BUTTON_FONT_SIZE,
            asset_manager,
            user_settings,
        )
    }

    fn handle_world_name_input(&mut self, width: f32, height: f32, font: &Font) {
        let world_selection_x = (width - TEXT_INPUT_SIZE.x) / 2.0;
        let world_selection_y = height * WORLD_NAME_INPUT_Y_COEF;

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
            font,
        );
        if !self.error.is_empty() {
            self.validate();
        }
    }

    fn draw_input_label(width: f32, height: f32, font: &Font) {
        let text = "Enter world name:";
        let x = (width - get_text_width(text, LABEL_FONT_SIZE as u16, font)) * 0.5;
        let y = height * 0.15;
        draw_game_text(text, x, y, LABEL_FONT_SIZE, TEXT_COLOR, font);
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
