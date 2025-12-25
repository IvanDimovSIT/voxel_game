use std::sync::{Arc, LazyLock};

use macroquad::prelude::info;

use crate::graphics::{flat_shader::FlatShader, sky_shader::SkyShader, voxel_shader::VoxelShader};

// global shader singleton
static SHADER_MANAGER_INSTANCE: LazyLock<Arc<ShaderManager>> = LazyLock::new(ShaderManager::new);

pub struct ShaderManager {
    pub voxel_shader: VoxelShader,
    pub sky_shader: SkyShader,
    pub flat_shader: FlatShader,
}
impl ShaderManager {
    pub fn instance() -> Arc<Self> {
        SHADER_MANAGER_INSTANCE.clone()
    }

    fn new() -> Arc<Self> {
        let shader_manager = Arc::new(Self {
            voxel_shader: VoxelShader::new(),
            sky_shader: SkyShader::new(),
            flat_shader: FlatShader::new(),
        });
        info!("Initialised shaders");

        shader_manager
    }
}
