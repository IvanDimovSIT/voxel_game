use std::rc::Rc;

use macroquad::text::{Font, load_ttf_font_from_bytes};

use crate::{
    graphics::{mesh_manager::MeshManager, texture_manager::TextureManager},
    service::sound_manager::SoundManager,
};

const FONT: &[u8; 42896] = include_bytes!("../../resources/font.ttf");

#[derive(Debug, Clone)]
pub enum AssetError {
    MissingImage { path: String },
    MissingSound { path: String },
    MissingModel { path: String },
    ModelFileMustContainASingleModel { path: String },
}

#[derive(Debug)]
pub struct AssetLoadingErrors {
    pub errors: Vec<AssetError>,
}
impl AssetLoadingErrors {
    pub fn new(errors: Vec<AssetError>) -> Self {
        Self { errors }
    }
}

pub struct AssetManager {
    pub texture_manager: TextureManager,
    pub sound_manager: SoundManager,
    pub mesh_manager: MeshManager,
    pub font: Font,
}
impl AssetManager {
    pub async fn new() -> Result<Rc<Self>, AssetLoadingErrors> {
        let mut errors = vec![];

        let font = load_ttf_font_from_bytes(FONT).expect("Error loading font");
        let texture_manager_result = TextureManager::new().await;
        let sound_manager_result = SoundManager::new().await;
        if let Err(image_errors) = &texture_manager_result {
            errors.extend(image_errors.errors.clone());
        }
        if let Err(sound_errors) = &sound_manager_result {
            errors.extend(sound_errors.errors.clone());
        }

        if errors.is_empty() {
            let texture_manager = texture_manager_result.unwrap();
            let sound_manager = sound_manager_result.unwrap();
            match MeshManager::new(&texture_manager) {
                Ok(mesh_manager) => {
                    return Ok(Rc::new(Self {
                        texture_manager,
                        sound_manager,
                        mesh_manager,
                        font,
                    }));
                }
                Err(model_errors) => {
                    errors.extend(model_errors.errors);
                }
            }
        }

        assert!(!errors.is_empty());
        Err(AssetLoadingErrors::new(errors))
    }
}
