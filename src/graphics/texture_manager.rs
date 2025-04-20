use std::collections::HashMap;

use macroquad::{
    logging::error,
    texture::{Texture2D, build_textures_atlas, load_texture},
};

use crate::model::voxel::Voxel;

const TEXTURES: [(Voxel, &str); 8] = [
    (Voxel::Stone, "resources/images/stone.png"),
    (Voxel::Sand, "resources/images/sand.png"),
    (Voxel::Grass, "resources/images/grass.png"),
    (Voxel::Wood, "resources/images/wood.png"),
    (Voxel::Leaves, "resources/images/leaves.png"),
    (Voxel::Brick, "resources/images/brick.png"),
    (Voxel::Dirt, "resources/images/dirt.png"),
    (Voxel::Boards, "resources/images/boards.png"),
];

pub struct TextureManager {
    textures: HashMap<Voxel, Texture2D>,
}
impl TextureManager {
    /// loads all of the textures
    pub async fn new() -> Self {
        let mut textures = HashMap::new();
        for (texture_type, texture_path) in TEXTURES {
            textures.insert(texture_type, Self::load_image(texture_path).await);
        }

        build_textures_atlas();
        Self { textures }
    }

    async fn load_image(path: &str) -> Texture2D {
        load_texture(path)
            .await
            .expect(&format!("Error loading texture '{path}'"))
    }

    /// returns the texture of the voxel
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
