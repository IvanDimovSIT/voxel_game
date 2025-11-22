use macroquad::{
    camera::Camera3D,
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

const FLAT_VERTEX_SHADER: &str = include_str!("../../resources/shaders/flat_vertex.glsl");
const FLAT_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/flat_fragment.glsl");

const CAMERA_POSITION_UNIFORM: &str = "cameraPos";

pub struct FlatShader {
    flat_material: Material,
}
impl FlatShader {
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

        let camera_uniform = UniformDesc::new(CAMERA_POSITION_UNIFORM, UniformType::Float3);

        let flat_material = load_material(
            ShaderSource::Glsl {
                vertex: FLAT_VERTEX_SHADER,
                fragment: FLAT_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: voxel_pipeline_params,
                uniforms: vec![camera_uniform],
                ..Default::default()
            },
        )
        .expect("Error initialising flat shaders");

        Self { flat_material }
    }

    pub fn set_flat_material(&self, camera: &Camera3D) {
        self.flat_material.set_uniform(
            CAMERA_POSITION_UNIFORM,
            [camera.position.x, camera.position.y, camera.position.z],
        );
        gl_use_material(&self.flat_material);
    }
}
