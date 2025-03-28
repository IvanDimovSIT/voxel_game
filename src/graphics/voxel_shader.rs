use macroquad::prelude::{
    gl_use_material, load_material, Comparison, Material, MaterialParams, PipelineParams, ShaderSource
};

pub const VERTEX_SHADER: &str = include_str!("../../resources/shaders/vertex.glsl");
pub const FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/fragment.glsl");

pub struct VoxelShader {
    material: Material,
}
impl VoxelShader {
    pub fn new() -> Self {
        let pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            cull_face: macroquad::miniquad::CullFace::Back,
            ..Default::default()
        };

        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params,
                ..Default::default()
            },
        )
        .expect("Error initialising shaders");

        Self { material }
    }

    pub fn set_material(&self) {
        gl_use_material(&self.material);
    }
}
