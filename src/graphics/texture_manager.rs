use std::collections::HashMap;

use macroquad::{
    prelude::{error, info},
    texture::{FilterMode, Texture2D, load_texture},
};

use crate::{
    graphics::mesh_manager::MeshId,
    model::voxel::{MAX_VOXEL_VARIANTS, Voxel},
};

const BASE_CREATURE_TEXTURES_PATH: &str = "assets/images/creature_textures/";
const BASE_VOXEL_TEXTURES_PATH: &str = "assets/images/voxels/";
const BASE_ICON_TEXTURES_PATH: &str = "assets/images/icons/";
const TEXTURES: [(Voxel, &str); 16] = [
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
    (Voxel::Snow, "snow.png"),
    (Voxel::Ice, "ice.png"),
];
const WATER_TEXTURE: &str = "water.png";
const ICON_TEXTURES: [(Voxel, &str); 4] = [
    (Voxel::Grass, "grass-icon.png"),
    (Voxel::Trampoline, "trampoline-icon.png"),
    (Voxel::Wood, "wood-icon.png"),
    (Voxel::Glass, "glass-icon.png"),
];
const MESH_TEXTURES: [(MeshId, &str); MeshId::VARIANTS] = [
    (MeshId::Bunny, "bunny_texture.png"),
    (MeshId::ButterflyDown, "butterfly_texture.png"),
    (MeshId::ButterflyUp, "butterfly_texture.png"),
    (MeshId::Penguin, "penguin_texture.png"),
];
const MAX_TEXTURE_COUNT: usize = MAX_VOXEL_VARIANTS;

const BASE_PLAIN_TEXTURES: &str = "assets/images/others/";
const PLAIN_TEXTURES: [(PlainTextureId, &str, FilterMode); 5] = [
    (PlainTextureId::Sun, "sun.png", FilterMode::Linear),
    (PlainTextureId::Moon, "moon.png", FilterMode::Linear),
    (PlainTextureId::Clouds, "clouds.png", FilterMode::Nearest),
    (
        PlainTextureId::TitleScreenBackground,
        "title.png",
        FilterMode::Linear,
    ),
    (PlainTextureId::Controls, "controls.png", FilterMode::Linear),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlainTextureId {
    Sun,
    Moon,
    Clouds,
    TitleScreenBackground,
    Controls,
}

pub struct TextureManager {
    textures: Vec<Option<Texture2D>>,
    mesh_textures: HashMap<MeshId, Texture2D>,
    voxel_icons: HashMap<Voxel, Texture2D>,
    plain_textures: HashMap<PlainTextureId, Texture2D>,
}
impl TextureManager {
    pub const VOXELS_WITH_DIFFERENT_FACES: [Voxel; 3] =
        [Voxel::Grass, Voxel::Trampoline, Voxel::Wood];

    /// loads all of the textures
    pub async fn new() -> Self {
        let textures = Self::load_voxel_textures().await;
        let voxel_icons = Self::load_voxel_icon_textures().await;
        let mesh_textures = Self::load_mesh_textures().await;
        let plain_textures = Self::load_plain_textures().await;

        Self {
            textures,
            voxel_icons,
            mesh_textures,
            plain_textures,
        }
    }

    async fn load_plain_textures() -> HashMap<PlainTextureId, Texture2D> {
        let mut textures = HashMap::with_capacity(PLAIN_TEXTURES.len());
        for (texture_type, texture_path, filter_mode) in PLAIN_TEXTURES {
            let full_path = format!("{BASE_PLAIN_TEXTURES}{texture_path}");
            let texture = Self::load_image(&full_path).await;
            texture.set_filter(filter_mode);
            textures.insert(texture_type, texture);
            info!(
                "Loaded plain texture for {:?} from '{}'",
                texture_type, texture_path
            );
        }

        textures
    }

    async fn load_voxel_icon_textures() -> HashMap<Voxel, Texture2D> {
        let mut textures = HashMap::with_capacity(ICON_TEXTURES.len());
        for (texture_type, texture_path) in ICON_TEXTURES {
            let full_path = format!("{BASE_ICON_TEXTURES_PATH}{texture_path}");
            let texture = Self::load_image(&full_path).await;
            textures.insert(texture_type, texture);
            info!(
                "Loaded icon texture for {:?} from '{}'",
                texture_type, texture_path
            );
        }
        Self::verify_loaded_textures_for_multiface_voxels(&textures);

        textures
    }

    fn verify_loaded_textures_for_multiface_voxels(textures: &HashMap<Voxel, Texture2D>) {
        for voxel in Self::VOXELS_WITH_DIFFERENT_FACES {
            assert!(textures.contains_key(&voxel))
        }
    }

    async fn load_voxel_textures() -> Vec<Option<Texture2D>> {
        let mut textures = vec![None; MAX_TEXTURE_COUNT];
        for (texture_type, texture_path) in TEXTURES {
            let full_path = format!("{BASE_VOXEL_TEXTURES_PATH}{texture_path}");
            let texture = Self::load_image(&full_path).await;
            texture.set_filter(FilterMode::Nearest);
            textures[texture_type.index()] = Some(texture);
            info!(
                "Loaded texture for {:?} from '{}'",
                texture_type, texture_path
            );
        }
        Self::load_water_texture(&mut textures).await;

        textures
    }

    /// loads the texture once and reuses it for all water voxels
    async fn load_water_texture(textures: &mut [Option<Texture2D>]) {
        let water_voxels = [
            Voxel::WaterSource,
            Voxel::WaterDown,
            Voxel::Water1,
            Voxel::Water2,
            Voxel::Water3,
            Voxel::Water4,
        ];
        let texture_path = format!("{BASE_VOXEL_TEXTURES_PATH}{WATER_TEXTURE}");
        let texture = Self::load_image(&texture_path).await;
        texture.set_filter(FilterMode::Nearest);
        for voxel in water_voxels {
            textures[voxel.index()] = Some(texture.clone());
        }
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

    pub fn get_plain_texture(&self, texture_id: PlainTextureId) -> Texture2D {
        self.plain_textures
            .get(&texture_id)
            .expect("Plain texture not found")
            .weak_clone()
    }

    /// returns the icon texture of the voxel, could be the same as the regular texture
    pub fn get_icon(&self, voxel: Voxel) -> Texture2D {
        self.voxel_icons
            .get(&voxel)
            .map(Texture2D::weak_clone)
            .unwrap_or_else(|| self.get(voxel))
    }

    pub fn get_mesh_texture(&self, id: MeshId) -> Texture2D {
        self.mesh_textures
            .get(&id)
            .expect("Creature texture not loaded")
            .weak_clone()
    }

    async fn load_mesh_textures() -> HashMap<MeshId, Texture2D> {
        let mut textures = HashMap::new();
        for (id, file) in MESH_TEXTURES {
            let fullpath = format!("{BASE_CREATURE_TEXTURES_PATH}{file}");
            let texture = load_texture(&fullpath)
                .await
                .unwrap_or_else(|_| panic!("Error loading creature texture '{fullpath}'"));
            texture.set_filter(FilterMode::Nearest);

            textures.insert(id, texture);
        }

        textures
    }
}
