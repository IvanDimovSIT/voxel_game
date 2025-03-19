use std::{collections::HashMap, fs::File, io::Read};

use macroquad::texture::{load_texture, Image, Texture2D};

use crate::model::voxel::{self, Voxel};


pub struct TextureManager {
    textures: HashMap<Voxel, Texture2D>
}
impl TextureManager {
    pub async fn new() -> Self {
        let mut textures = HashMap::new();
        textures.insert(Voxel::Stone, Self::load_image("resources/images/stone.png").await);
        Self { textures }
    }

    async fn load_image(path: &str) -> Texture2D {
        load_texture(path).await.expect("Error loading texture '{path}'")
    }

    pub fn get(&self, voxel: Voxel) -> Texture2D {
        self.textures.get(&voxel)
            .or_else(|| {
                println!("No texture for {voxel:?} found. Using any texture.");   
                self.textures.values().nth(0)
        }).expect("No textures loaded").clone()
    }
}