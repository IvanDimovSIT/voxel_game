use std::collections::HashMap;

use macroquad::{
    prelude::{error, info},
    texture::{FilterMode, Texture2D, load_texture},
};

use crate::{
    graphics::mesh_manager::MeshId,
    model::voxel::{MAX_VOXEL_VARIANTS, Voxel},
};

const BASE_CREATURE_TEXTURES_PATH: &str = "assets/images/";
const BASE_VOXEL_TEXTURES_PATH: &str = "assets/images/voxels/";
const BASE_ICON_TEXTURES_PATH: &str = "assets/images/icons/";
const TITLE_SCREEN_BACKGROUND_PATH: &str = "assets/images/title.png";
const SUN_PATH: &str = "assets/images/sun.png";
const MOON_PATH: &str = "assets/images/moon.png";
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
const ICON_TEXTURES: [(Voxel, &str); 3] = [
    (Voxel::Grass, "grass-icon.png"),
    (Voxel::Trampoline, "trampoline-icon.png"),
    (Voxel::Wood, "wood-icon.png"),
];
const MESH_TEXTURES: [(MeshId, &str); MeshId::VARIANTS] = [
    (MeshId::Bunny, "bunny_texture.png"),
    (MeshId::ButterflyDown, "butterfly_texture.png"),
    (MeshId::ButterflyUp, "butterfly_texture.png"),
    (MeshId::Penguin, "penguin_texture.png"),
];
const MAX_TEXTURE_COUNT: usize = MAX_VOXEL_VARIANTS;

pub struct TextureManager {
    textures: Vec<Option<Texture2D>>,
    mesh_textures: HashMap<MeshId, Texture2D>,
    title_screen_background: Texture2D,
    sun_texture: Texture2D,
    moon_texture: Texture2D,
    voxel_icons: HashMap<Voxel, Texture2D>,
}
impl TextureManager {
    pub const VOXELS_WITH_DIFFERENT_FACES: [Voxel; 3] =
        [Voxel::Grass, Voxel::Trampoline, Voxel::Wood];

    /// loads all of the textures
    pub async fn new() -> Self {
        let textures = Self::load_voxel_textures().await;
        let title_screen_background = Self::load_image(TITLE_SCREEN_BACKGROUND_PATH).await;
        let sun_texture = Self::load_image(SUN_PATH).await;
        let moon_texture = Self::load_image(MOON_PATH).await;
        let voxel_icons = Self::load_voxel_icon_textures().await;
        let mesh_textures = Self::load_mesh_textures().await;

        Self {
            textures,
            title_screen_background,
            voxel_icons,
            sun_texture,
            moon_texture,
            mesh_textures,
        }
    }

    async fn load_voxel_icon_textures() -> HashMap<Voxel, Texture2D> {
        let mut textures = HashMap::with_capacity(Self::VOXELS_WITH_DIFFERENT_FACES.len());
        for (texture_type, texture_path) in ICON_TEXTURES {
            assert!(Self::VOXELS_WITH_DIFFERENT_FACES.contains(&texture_type));
            let full_path = format!("{BASE_ICON_TEXTURES_PATH}{texture_path}");
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

    pub fn get_sun_texture(&self) -> Texture2D {
        self.sun_texture.weak_clone()
    }

    pub fn get_moon_texture(&self) -> Texture2D {
        self.moon_texture.weak_clone()
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
