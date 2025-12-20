use std::collections::HashMap;

use macroquad::{
    prelude::{error, info},
    texture::{FilterMode, Texture2D, load_texture},
};

use crate::{
    graphics::mesh_manager::MeshId,
    model::voxel::{MAX_VOXEL_VARIANTS, Voxel},
    service::asset_manager::{AssetError, AssetLoadingErrors},
};

const BASE_MODEL_TEXTURES_PATH: &str = "assets/images/model_textures/";
const BASE_VOXEL_TEXTURES_PATH: &str = "assets/images/voxels/";
const BASE_ICON_TEXTURES_PATH: &str = "assets/images/icons/";
const TEXTURES: [(Voxel, &str); 19] = [
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
    (Voxel::StoneBrick, "stone-brick.png"),
    (Voxel::StonePillar, "stone-pillar.png"),
    (Voxel::Bomb, "bomb.png"),
];
const WATER_TEXTURE: &str = "water.png";
const ICON_TEXTURES: [(Voxel, &str); 6] = [
    (Voxel::Grass, "grass-icon.png"),
    (Voxel::Trampoline, "trampoline-icon.png"),
    (Voxel::Wood, "wood-icon.png"),
    (Voxel::Glass, "glass-icon.png"),
    (Voxel::StonePillar, "stone-pillar-icon.png"),
    (Voxel::Bomb, "bomb-icon.png"),
];
const MESH_TEXTURES: [(MeshId, &str); MeshId::VARIANTS] = [
    (MeshId::Bunny, "bunny_texture.png"),
    (MeshId::ButterflyDown, "butterfly_texture.png"),
    (MeshId::ButterflyUp, "butterfly_texture.png"),
    (MeshId::Penguin, "penguin_texture.png"),
];
const MAX_TEXTURE_COUNT: usize = MAX_VOXEL_VARIANTS;

const BASE_PLAIN_TEXTURES: &str = "assets/images/others/";
const PLAIN_TEXTURES: [(PlainTextureId, &str, FilterMode); 6] = [
    (PlainTextureId::Sun, "sun.png", FilterMode::Linear),
    (PlainTextureId::Moon, "moon.png", FilterMode::Linear),
    (PlainTextureId::Clouds, "clouds.png", FilterMode::Nearest),
    (
        PlainTextureId::TitleScreenBackground,
        "title.png",
        FilterMode::Linear,
    ),
    (PlainTextureId::Controls, "controls.png", FilterMode::Linear),
    (
        PlainTextureId::Lightning,
        "lightning.png",
        FilterMode::Linear,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlainTextureId {
    Sun,
    Moon,
    Clouds,
    TitleScreenBackground,
    Controls,
    Lightning,
}

pub struct TextureManager {
    textures: Vec<Option<Texture2D>>,
    mesh_textures: HashMap<MeshId, Texture2D>,
    voxel_icons: HashMap<Voxel, Texture2D>,
    plain_textures: HashMap<PlainTextureId, Texture2D>,
}
impl TextureManager {
    pub const VOXELS_WITH_DIFFERENT_FACES: [Voxel; 5] = [
        Voxel::Grass,
        Voxel::Trampoline,
        Voxel::Wood,
        Voxel::StonePillar,
        Voxel::Bomb,
    ];

    /// loads all of the textures
    pub async fn new() -> Result<Self, AssetLoadingErrors> {
        let textures = Self::load_voxel_textures().await;
        let voxel_icons = Self::load_voxel_icon_textures().await;
        let mesh_textures = Self::load_mesh_textures().await;
        let plain_textures = Self::load_plain_textures().await;

        let mut errors = vec![];

        if let Err(err) = &textures {
            errors.extend(err.errors.clone());
        }
        if let Err(err) = &voxel_icons {
            errors.extend(err.errors.clone());
        }
        if let Err(err) = &mesh_textures {
            errors.extend(err.errors.clone());
        }
        if let Err(err) = &plain_textures {
            errors.extend(err.errors.clone());
        }

        if !errors.is_empty() {
            return Err(AssetLoadingErrors::new(errors));
        }

        Ok(Self {
            textures: textures.unwrap(),
            voxel_icons: voxel_icons.unwrap(),
            mesh_textures: mesh_textures.unwrap(),
            plain_textures: plain_textures.unwrap(),
        })
    }

    async fn load_plain_textures() -> Result<HashMap<PlainTextureId, Texture2D>, AssetLoadingErrors>
    {
        let mut textures = HashMap::with_capacity(PLAIN_TEXTURES.len());
        let mut errors = vec![];

        for (texture_type, texture_path, filter_mode) in PLAIN_TEXTURES {
            let full_path = format!("{BASE_PLAIN_TEXTURES}{texture_path}");
            if let Some(texture) = Self::load_image(&full_path).await {
                texture.set_filter(filter_mode);
                textures.insert(texture_type, texture);
                info!(
                    "Loaded plain texture for {:?} from '{}'",
                    texture_type, texture_path
                );
            } else {
                errors.push(AssetError::MissingImage { path: full_path });
            }
        }

        if errors.is_empty() {
            Ok(textures)
        } else {
            Err(AssetLoadingErrors::new(errors))
        }
    }

    async fn load_voxel_icon_textures() -> Result<HashMap<Voxel, Texture2D>, AssetLoadingErrors> {
        let mut textures = HashMap::with_capacity(ICON_TEXTURES.len());
        let mut errors = vec![];

        for (texture_type, texture_path) in ICON_TEXTURES {
            let full_path = format!("{BASE_ICON_TEXTURES_PATH}{texture_path}");
            if let Some(texture) = Self::load_image(&full_path).await {
                textures.insert(texture_type, texture);
                info!(
                    "Loaded icon texture for {:?} from '{}'",
                    texture_type, texture_path
                );
            } else {
                errors.push(AssetError::MissingImage { path: full_path });
            }
        }

        if errors.is_empty() {
            Self::verify_loaded_textures_for_multiface_voxels(&textures);
            Ok(textures)
        } else {
            Err(AssetLoadingErrors::new(errors))
        }
    }

    fn verify_loaded_textures_for_multiface_voxels(textures: &HashMap<Voxel, Texture2D>) {
        for voxel in Self::VOXELS_WITH_DIFFERENT_FACES {
            assert!(textures.contains_key(&voxel))
        }
    }

    async fn load_voxel_textures() -> Result<Vec<Option<Texture2D>>, AssetLoadingErrors> {
        let mut textures = vec![None; MAX_TEXTURE_COUNT];
        let mut missing_textures = vec![];

        for (texture_type, texture_path) in TEXTURES {
            let full_path = format!("{BASE_VOXEL_TEXTURES_PATH}{texture_path}");
            if let Some(texture) = Self::load_image(&full_path).await {
                texture.set_filter(FilterMode::Nearest);
                textures[texture_type.index()] = Some(texture);
                info!(
                    "Loaded texture for {:?} from '{}'",
                    texture_type, texture_path
                );
            } else {
                missing_textures.push(AssetError::MissingImage { path: full_path });
            }
        }
        if let Err(missing) = Self::load_water_texture(&mut textures).await {
            missing_textures.push(missing);
        }

        if missing_textures.is_empty() {
            Ok(textures)
        } else {
            Err(AssetLoadingErrors::new(missing_textures))
        }
    }

    /// loads the texture once and reuses it for all water voxels
    async fn load_water_texture(textures: &mut [Option<Texture2D>]) -> Result<(), AssetError> {
        let water_voxels = [
            Voxel::WaterSource,
            Voxel::WaterDown,
            Voxel::Water1,
            Voxel::Water2,
            Voxel::Water3,
            Voxel::Water4,
        ];
        let texture_path = format!("{BASE_VOXEL_TEXTURES_PATH}{WATER_TEXTURE}");
        if let Some(texture) = Self::load_image(&texture_path).await {
            texture.set_filter(FilterMode::Nearest);
            for voxel in water_voxels {
                textures[voxel.index()] = Some(texture.clone());
            }
            Ok(())
        } else {
            Err(AssetError::MissingImage { path: texture_path })
        }
    }

    async fn load_image(path: &str) -> Option<Texture2D> {
        match load_texture(path).await {
            Ok(texture) => Some(texture),
            Err(err) => {
                error!("Error loading texture '{}':{}", path, err);
                None
            }
        }
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

    async fn load_mesh_textures() -> Result<HashMap<MeshId, Texture2D>, AssetLoadingErrors> {
        let mut textures = HashMap::new();
        let mut errors = vec![];

        for (id, file) in MESH_TEXTURES {
            let fullpath = format!("{BASE_MODEL_TEXTURES_PATH}{file}");
            if let Some(texture) = Self::load_image(&fullpath).await {
                texture.set_filter(FilterMode::Nearest);
                textures.insert(id, texture);
            } else {
                errors.push(AssetError::MissingImage { path: fullpath });
            }
        }

        if errors.is_empty() {
            Ok(textures)
        } else {
            Err(AssetLoadingErrors::new(errors))
        }
    }
}
