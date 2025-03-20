use std::collections::HashMap;

use macroquad::{
    logging::error,
    texture::{Texture2D, load_texture},
};

use crate::model::voxel::Voxel;

pub struct TextureManager {
    textures: HashMap<Voxel, Texture2D>,
}
impl TextureManager {
    pub async fn new() -> Self {
        let mut textures = HashMap::new();
        textures.insert(
            Voxel::Stone,
            Self::load_image("resources/images/stone.png").await,
        );
        Self { textures }
    }

    async fn load_image(path: &str) -> Texture2D {
        load_texture(path)
            .await
            .expect("Error loading texture '{path}'")
    }

    pub fn get(&self, voxel: Voxel) -> Texture2D {
        self.textures
            .get(&voxel)
            .or_else(|| {
                error!("No texture for {:?} found. Using any texture.", voxel);
                self.textures.values().nth(0)
            })
            .expect("No textures loaded")
            .clone()
    }
}
