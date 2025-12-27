use std::sync::LazyLock;

use macroquad::prelude::info;

use crate::graphics::{flat_shader::FlatShader, sky_shader::SkyShader, voxel_shader::VoxelShader};

/// global shader singleton containing all game shaders
pub static SHADER_MANAGER_INSTANCE: LazyLock<ShaderManager> = LazyLock::new(ShaderManager::new);

pub struct ShaderManager {
    pub voxel_shader: VoxelShader,
    pub sky_shader: SkyShader,
    pub flat_shader: FlatShader,
}
impl ShaderManager {
    pub fn initialise_global_instance() {
        LazyLock::force(&SHADER_MANAGER_INSTANCE);
    }

    fn new() -> Self {
        let shader_manager = Self {
            voxel_shader: VoxelShader::new(),
            sky_shader: SkyShader::new(),
            flat_shader: FlatShader::new(),
        };
        info!("Initialised shaders");

        shader_manager
    }
}
