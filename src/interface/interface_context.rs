use std::rc::Rc;

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{
        settings_menu::SettingsContext, title_screen::TitleScreenContext,
        world_selection::WorldSelectionContext,
    },
    model::user_settings::UserSettings,
    service::sound_manager::SoundManager,
    voxel_engine::VoxelEngine,
};

pub enum InterfaceScreen {
    WorldSelection(WorldSelectionContext),
    TitleScreen(TitleScreenContext),
    Settings(SettingsContext),
}

pub struct InterfaceContext {
    current_screen: InterfaceScreen,
    sound_manager: Rc<SoundManager>,
    texture_manager: Rc<TextureManager>,
    user_settings: UserSettings,
}
impl InterfaceContext {
    pub fn new_world_selection(
        sound_manager: Rc<SoundManager>,
        texture_manager: Rc<TextureManager>,
        user_settings: UserSettings,
    ) -> Self {
        Self {
            current_screen: InterfaceScreen::WorldSelection(WorldSelectionContext::new()),
            sound_manager,
            user_settings,
            texture_manager,
        }
    }

    pub fn new_title_screen(
        sound_manager: Rc<SoundManager>,
        texture_manager: Rc<TextureManager>,
        user_settings: UserSettings,
    ) -> Self {
        Self {
            current_screen: InterfaceScreen::TitleScreen(TitleScreenContext::new()),
            sound_manager,
            user_settings,
            texture_manager,
        }
    }

    pub fn enter_game(&self) -> Option<Box<VoxelEngine>> {
        match &self.current_screen {
            InterfaceScreen::WorldSelection(world_selection_context) => world_selection_context
                .enter_game(
                    self.texture_manager.clone(),
                    self.sound_manager.clone(),
                    &self.user_settings,
                ),
            _ => None,
        }
    }

    pub async fn draw(&mut self) {
        match &mut self.current_screen {
            InterfaceScreen::WorldSelection(world_selection_context) => {
                let new_screen = world_selection_context
                    .draw(
                        &self.texture_manager,
                        &self.sound_manager,
                        &self.user_settings,
                    )
                    .await;

                if let Some(screen) = new_screen {
                    self.current_screen = screen;
                }
            }
            InterfaceScreen::TitleScreen(title_screen_context) => {
                let new_screen = title_screen_context
                    .draw(
                        &self.texture_manager,
                        &self.sound_manager,
                        &self.user_settings,
                    )
                    .await;

                self.current_screen = new_screen;
            }
            InterfaceScreen::Settings(settings_context) => {
                let new_screen = settings_context
                    .draw(
                        &self.texture_manager,
                        &self.sound_manager,
                        &mut self.user_settings,
                    )
                    .await;
                self.current_screen = new_screen;
            }
        }
    }

    pub fn should_exit(&self) -> bool {
        match &self.current_screen {
            InterfaceScreen::TitleScreen(title_screen_context) => {
                title_screen_context.should_exit()
            }
            _ => false,
        }
    }
}
