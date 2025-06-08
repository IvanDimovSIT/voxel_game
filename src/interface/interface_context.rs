use std::rc::Rc;

use crate::{
    graphics::texture_manager::TextureManager,
    interface::{title_screen::TitleScreenContext, world_selection::WorldSelectionContext},
    model::user_settings::UserSettings,
    service::sound_manager::SoundManager,
    voxel_engine::VoxelEngine,
};

enum InterfaceScreen {
    WorldSelection(WorldSelectionContext),
    TitleScreen(TitleScreenContext),
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
                world_selection_context
                    .draw(
                        &self.texture_manager,
                        &self.sound_manager,
                        &self.user_settings,
                    )
                    .await;
                if world_selection_context.should_go_to_title() {
                    self.current_screen = InterfaceScreen::TitleScreen(TitleScreenContext::new())
                }
            }
            InterfaceScreen::TitleScreen(title_screen_context) => {
                title_screen_context
                    .draw(
                        &self.texture_manager,
                        &self.sound_manager,
                        &self.user_settings,
                    )
                    .await;
                if title_screen_context.should_play() {
                    self.current_screen =
                        InterfaceScreen::WorldSelection(WorldSelectionContext::new())
                }
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
