use std::collections::HashMap;

use macroquad::{
    logging::error,
    texture::{Texture2D, build_textures_atlas, load_texture},
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
        textures.insert(
            Voxel::Sand,
            Self::load_image("resources/images/sand.png").await,
        );
        textures.insert(
            Voxel::Grass,
            Self::load_image("resources/images/grass.png").await,
        );
        textures.insert(
            Voxel::Wood,
            Self::load_image("resources/images/wood.png").await,
        );
        textures.insert(
            Voxel::Leaves,
            Self::load_image("resources/images/leaves.png").await,
        );
        build_textures_atlas();
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
