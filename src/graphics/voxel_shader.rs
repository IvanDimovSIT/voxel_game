use macroquad::{
    camera::Camera3D,
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

const VOXEL_VERTEX_SHADER: &str = include_str!("../../resources/shaders/voxel_vertex.glsl");
const VOXEL_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/voxel_fragment.glsl");

/// default 3D material shader for voxels
pub struct VoxelShader {
    voxel_material: Material,
}
impl VoxelShader {
    pub fn new() -> Self {
        let voxel_pipeline_params = PipelineParams {
            depth_write: true,
            depth_test: Comparison::LessOrEqual,
            cull_face: macroquad::miniquad::CullFace::Back,
            ..Default::default()
        };

        let camera_uniform = UniformDesc::new("cameraPos", UniformType::Float3);
        let look_uniform = UniformDesc::new("cameraTarget", UniformType::Float3);

        let voxel_material = load_material(
            ShaderSource::Glsl {
                vertex: VOXEL_VERTEX_SHADER,
                fragment: VOXEL_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: voxel_pipeline_params,
                uniforms: vec![camera_uniform, look_uniform],
                ..Default::default()
            },
        )
        .expect("Error initialising voxel shaders");

        Self { voxel_material }
    }

    /// sets the current OpenGL shader to render the world voxels
    pub fn set_voxel_material(&self, camera: &Camera3D) {
        self.voxel_material.set_uniform(
            "cameraPos",
            [camera.position.x, camera.position.y, camera.position.z],
        );
        self.voxel_material.set_uniform(
            "cameraTarget",
            [camera.target.x, camera.target.y, camera.target.z],
        );

        gl_use_material(&self.voxel_material);
    }
}
