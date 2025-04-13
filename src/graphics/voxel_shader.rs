use macroquad::{
    camera::Camera3D,
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

pub const VERTEX_SHADER: &str = include_str!("../../resources/shaders/vertex.glsl");
pub const FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/fragment.glsl");

/// 3D material shader
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

        let camera_uniform = UniformDesc::new("cameraPos", UniformType::Float3);
        let look_uniform = UniformDesc::new("cameraTarget", UniformType::Float3);

        let material = load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params,
                uniforms: vec![camera_uniform, look_uniform],
                ..Default::default()
            },
        )
        .expect("Error initialising shaders");

        Self { material }
    }

    pub fn set_material(&self, camera: &Camera3D) {
        self.material.set_uniform(
            "cameraPos",
            [camera.position.x, camera.position.y, camera.position.z],
        );
        self.material.set_uniform(
            "cameraTarget",
            [camera.target.x, camera.target.y, camera.target.z],
        );

        gl_use_material(&self.material);
    }
}
