use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, gl_use_material,
        load_material,
    },
};

const SKY_VERTEX_SHADER: &str = include_str!("../../resources/shaders/sky_vertex.glsl");
const SKY_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/sky_fragment.glsl");

pub struct SkyShader {
    sky_material: Material,
}
impl SkyShader {
    pub fn new() -> Self {
        let voxel_pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let sky_material = load_material(
            ShaderSource::Glsl {
                vertex: SKY_VERTEX_SHADER,
                fragment: SKY_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: voxel_pipeline_params,
                ..Default::default()
            },
        )
        .expect("Error initialising sky shaders");

        Self { sky_material }
    }

    pub fn set_sky_material(&self) {
        gl_use_material(&self.sky_material);
    }
}
