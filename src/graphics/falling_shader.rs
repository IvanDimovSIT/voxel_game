use macroquad::{
    camera::Camera3D,
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

const FALLING_VERTEX_SHADER: &str = include_str!("../../resources/shaders/falling_vertex.glsl");
const FALLING_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/falling_fragment.glsl");

const CAMERA_POSITION_UNIFORM: &str = "cameraPos";

/// 3D material shader for falling voxels
pub struct FallingShader {
    falling_material: Material,
}
impl FallingShader {
    pub fn new() -> Self {
        let falling_pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            ..Default::default()
        };

        let camera_uniform = UniformDesc::new(CAMERA_POSITION_UNIFORM, UniformType::Float3);
        let falling_material = load_material(
            ShaderSource::Glsl {
                vertex: FALLING_VERTEX_SHADER,
                fragment: FALLING_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: falling_pipeline_params,
                uniforms: vec![camera_uniform],
                ..Default::default()
            },
        )
        .expect("Error initialising falling shaders");

        Self { falling_material }
    }

    /// sets the current OpenGL shader to render falling voxels
    pub fn set_falling_material(&self, camera: &Camera3D) {
        self.falling_material.set_uniform(
            CAMERA_POSITION_UNIFORM,
            [camera.position.x, camera.position.y, camera.position.z],
        );
        gl_use_material(&self.falling_material);
    }
}
