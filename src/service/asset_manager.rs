use std::rc::Rc;

use macroquad::text::{Font, load_ttf_font_from_bytes};

use crate::{
    graphics::{mesh_manager::MeshManager, texture_manager::TextureManager},
    service::sound_manager::SoundManager,
};

const FONT: &[u8; 42896] = include_bytes!("../../resources/font.ttf");

pub struct AssetManager {
    pub texture_manager: TextureManager,
    pub sound_manager: SoundManager,
    pub mesh_manager: MeshManager,
    pub font: Font,
}
impl AssetManager {
    pub async fn new() -> Rc<Self> {
        let texture_manager = TextureManager::new().await;
        let sound_manager = SoundManager::new().await;
        let font = load_ttf_font_from_bytes(FONT).expect("Error loading font");
        let mesh_manager = MeshManager::new(&texture_manager);

        Rc::new(Self {
            texture_manager,
            sound_manager,
            mesh_manager,
            font,
        })
    }
}
