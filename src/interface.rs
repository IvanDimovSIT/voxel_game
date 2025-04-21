use std::rc::Rc;

use macroquad::{
    camera::set_default_camera,
    color::{Color, DARKGRAY, RED},
    math::{RectOffset, Vec2, vec2},
    miniquad::window::screen_size,
    ui::{
        Skin, Style, hash, root_ui,
        widgets::{self, Group},
    },
    window::{clear_background, next_frame},
};

use crate::{
    graphics::texture_manager::{self, TextureManager},
    service::sound_manager::SoundManager,
    voxel_engine::VoxelEngine,
};

const WORLD_NAME_ID: u64 = 1;
const MENU_SIZE: Vec2 = vec2(500.0, 300.0);
const NEUTRAL_COLOR: Color = Color::from_rgba(220, 220, 220, 255);
const SELECTED_COLOR: Color = Color::from_rgba(230, 230, 255, 255);
const MESSAGE_POSITION: Vec2 = vec2(0.0, 80.0);

pub struct InterfaceContext {
    skin: Skin,
    world_name: String,
    error: String,
    should_enter: bool,
}
impl InterfaceContext {
    pub fn new() -> Self {
        Self {
            skin: Self::initialise_skin(),
            world_name: "".to_owned(),
            error: "".to_owned(),
            should_enter: false,
        }
    }

    pub async fn enter_game(
        &self,
        texture_manager: Rc<TextureManager>,
        sound_manager: Rc<SoundManager>,
    ) -> Option<Box<VoxelEngine>> {
        if self.should_enter {
            let voxel_engine = Box::new(VoxelEngine::new(
                &self.world_name,
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
        clear_background(DARKGRAY);
        let (width, height) = screen_size();

        root_ui().push_skin(&self.skin);
        widgets::Window::new(
            hash!(),
            vec2((width - MENU_SIZE.x) / 2.0, height * 0.1),
            MENU_SIZE,
        )
        .label("Voxel Game")
        .movable(false)
        .titlebar(true)
        .ui(&mut *root_ui(), |ui| {
            ui.input_text(WORLD_NAME_ID, "World name", &mut self.world_name);
            if ui.button(vec2(MENU_SIZE.x * 0.5, 150.0), "Play".to_owned()) {
                self.validate();
                if self.error.is_empty() {
                    self.should_enter = true;
                    ui.label(MESSAGE_POSITION, "Loading...");
                }
            }

            ui.label(MESSAGE_POSITION, &self.error);
        });

        self.remove_invalid_characters();
        if !self.error.is_empty() {
            self.validate();
        }
        root_ui().pop_skin();
        next_frame().await;
    }

    fn initialise_skin() -> Skin {
        let window_style = root_ui()
            .style_builder()
            .font_size(20)
            .margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
            .background_margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
            .build();

        let text_style = root_ui()
            .style_builder()
            .font_size(20)
            .margin(RectOffset::new(0.0, 0.0, 0.0, 10.0))
            .background_margin(RectOffset::new(0.0, 0.0, 10.0, 10.0))
            .color(NEUTRAL_COLOR)
            .color_selected(SELECTED_COLOR)
            .color_hovered(SELECTED_COLOR)
            .color_selected_hovered(SELECTED_COLOR)
            .color_clicked(SELECTED_COLOR)
            .build();

        let button_style = root_ui()
            .style_builder()
            .font_size(35)
            .margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
            .background_margin(RectOffset::new(10.0, 10.0, 10.0, 10.0))
            .color(NEUTRAL_COLOR)
            .color_hovered(SELECTED_COLOR)
            .build();

        Skin {
            window_style,
            button_style,
            label_style: text_style.clone(),
            editbox_style: text_style,
            ..root_ui().default_skin()
        }
    }

    fn validate(&mut self) {
        if self.world_name.len() < 3 {
            self.error = "World name should be at least 3 characters".to_owned();
        } else {
            self.error = "".to_owned();
        }
    }

    fn remove_invalid_characters(&mut self) {
        self.world_name = self
            .world_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }
}
