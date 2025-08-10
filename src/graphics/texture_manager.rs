use std::collections::HashMap;

use macroquad::{
    prelude::{error, info},
    texture::{FilterMode, Texture2D, build_textures_atlas, load_texture},
};

use crate::model::voxel::{MAX_VOXEL_VARIANTS, Voxel};

const BASE_TEXTURES_PATH: &str = "resources/images/";
const TITLE_SCREEN_BACKGROUND_PATH: &str = "resources/images/title.png";
const TEXTURES: [(Voxel, &str); 14] = [
    (Voxel::Stone, "stone.png"),
    (Voxel::Sand, "sand.png"),
    (Voxel::Grass, "grass.png"),
    (Voxel::Wood, "wood.png"),
    (Voxel::Leaves, "leaves.png"),
    (Voxel::Brick, "brick.png"),
    (Voxel::Dirt, "dirt.png"),
    (Voxel::Boards, "boards.png"),
    (Voxel::Cobblestone, "cobblestone.png"),
    (Voxel::Clay, "clay.png"),
    (Voxel::Lamp, "lamp.png"),
    (Voxel::Trampoline, "trampoline.png"),
    (Voxel::Glass, "glass.png"),
    (Voxel::Cactus, "cactus.png"),
];
const ICON_TEXTURES: [(Voxel, &str); 3] = [
    (Voxel::Grass, "grass-icon.png"),
    (Voxel::Trampoline, "trampoline-icon.png"),
    (Voxel::Wood, "wood-icon.png"),
];
const MAX_TEXTURE_COUNT: usize = MAX_VOXEL_VARIANTS;

pub struct TextureManager {
    textures: Vec<Option<Texture2D>>,
    title_screen_background: Texture2D,
    voxel_icons: HashMap<Voxel, Texture2D>,
}
impl TextureManager {
    pub const VOXELS_WITH_DIFFERENT_FACES: [Voxel; 3] =
        [Voxel::Grass, Voxel::Trampoline, Voxel::Wood];

    /// loads all of the textures
    pub async fn new() -> Self {
        let textures = Self::load_voxel_textures().await;
        let title_screen_background = Self::load_image(TITLE_SCREEN_BACKGROUND_PATH).await;
        let voxel_icons = Self::load_voxel_icon_textures().await;

        Self {
            textures,
            title_screen_background,
            voxel_icons,
        }
    }

    async fn load_voxel_icon_textures() -> HashMap<Voxel, Texture2D> {
        let mut textures = HashMap::with_capacity(Self::VOXELS_WITH_DIFFERENT_FACES.len());
        for (texture_type, texture_path) in ICON_TEXTURES {
            assert!(Self::VOXELS_WITH_DIFFERENT_FACES.contains(&texture_type));
            let full_path = format!("{BASE_TEXTURES_PATH}{texture_path}");
            let texture = Self::load_image(&full_path).await;
            textures.insert(texture_type, texture);
            info!(
                "Loaded icon texture for {:?} from '{}'",
                texture_type, texture_path
            );
        }

        textures
    }

    async fn load_voxel_textures() -> Vec<Option<Texture2D>> {
        let mut textures = vec![None; MAX_TEXTURE_COUNT];
        for (texture_type, texture_path) in TEXTURES {
            let full_path = format!("{BASE_TEXTURES_PATH}{texture_path}");
            let texture = Self::load_image(&full_path).await;
            texture.set_filter(FilterMode::Nearest);
            textures[texture_type.index()] = Some(texture);
            info!(
                "Loaded texture for {:?} from '{}'",
                texture_type, texture_path
            );
        }
        build_textures_atlas();

        textures
    }

    async fn load_image(path: &str) -> Texture2D {
        load_texture(path)
            .await
            .unwrap_or_else(|_| panic!("Error loading texture '{path}'"))
    }

    /// returns the texture of the voxel
    pub fn get(&self, voxel: Voxel) -> Texture2D {
        if let Some(texture) = &self.textures[voxel.index()] {
            texture.weak_clone()
        } else {
            error!("No texture loaded for {:?}", voxel);
            self.textures
                .iter()
                .flatten()
                .next()
                .cloned()
                .expect("No textures loaded")
        }
    }

    pub fn get_icon(&self, voxel: Voxel) -> Texture2D {
        if !Self::VOXELS_WITH_DIFFERENT_FACES.contains(&voxel) {
            return self.get(voxel);
        }

        self.voxel_icons
            .get(&voxel)
            .expect("Icon texture not loaded")
            .clone()
    }

    pub fn get_title_screen_background(&self) -> Texture2D {
        self.title_screen_background.weak_clone()
    }
}
