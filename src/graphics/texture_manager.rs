use macroquad::{
    prelude::{error, info},
    texture::{Texture2D, build_textures_atlas, load_texture},
};

use crate::model::voxel::{MAX_VOXEL_VARIANTS, Voxel};

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
const MAX_TEXTURE_COUNT: usize = MAX_VOXEL_VARIANTS;

pub struct TextureManager {
    textures: Vec<Option<Texture2D>>,
}
impl TextureManager {
    /// loads all of the textures
    pub async fn new() -> Self {
        let mut textures = vec![None; MAX_TEXTURE_COUNT];
        for (texture_type, texture_path) in TEXTURES {
            textures[texture_type.index()] = Some(Self::load_image(texture_path).await);
            info!(
                "Loaded texture for {:?} from '{}'",
                texture_type, texture_path
            );
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
        self.textures[voxel.index()]
            .clone()
            .or_else(|| {
                error!("No texture loaded for {:?}", voxel);
                self.textures.iter().flatten().next().cloned()
            })
            .expect("No textures loaded")
    }
}
