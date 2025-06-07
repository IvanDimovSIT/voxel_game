use macroquad::{
    camera::Camera3D,
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

use crate::{model::area::AREA_SIZE, service::world_time::WorldTime};

const VOXEL_VERTEX_SHADER: &str = include_str!("../../resources/shaders/voxel_vertex.glsl");
const VOXEL_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/voxel_fragment.glsl");

const CAMERA_POSITION_UNIFORM: &str = "cameraPos";
const CAMERA_TARGET_UNIFORM: &str = "cameraTarget";
const FOG_NEAR_UNIFORM: &str = "fogNear";
const FOG_FAR_UNIFORM: &str = "fogFar";
const LIGHT_LEVEL_UNIFORM: &str = "lightLevel";

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

        let camera_uniform = UniformDesc::new(CAMERA_POSITION_UNIFORM, UniformType::Float3);
        let look_uniform = UniformDesc::new(CAMERA_TARGET_UNIFORM, UniformType::Float3);
        let fog_near_uniform = UniformDesc::new(FOG_NEAR_UNIFORM, UniformType::Float1);
        let fog_far_uniform = UniformDesc::new(FOG_FAR_UNIFORM, UniformType::Float1);
        let light_level_uniform = UniformDesc::new(LIGHT_LEVEL_UNIFORM, UniformType::Float1);

        let voxel_material = load_material(
            ShaderSource::Glsl {
                vertex: VOXEL_VERTEX_SHADER,
                fragment: VOXEL_FRAGMENT_SHADER,
            },
            MaterialParams {
                pipeline_params: voxel_pipeline_params,
                uniforms: vec![
                    camera_uniform,
                    look_uniform,
                    fog_near_uniform,
                    fog_far_uniform,
                    light_level_uniform,
                ],
                ..Default::default()
            },
        )
        .expect("Error initialising voxel shaders");

        Self { voxel_material }
    }

    /// sets the current OpenGL shader to render the world voxels
    pub fn set_voxel_material(&self, camera: &Camera3D, render_size: u32, world_time: &WorldTime) {
        self.voxel_material.set_uniform(
            CAMERA_POSITION_UNIFORM,
            [camera.position.x, camera.position.y, camera.position.z],
        );
        self.voxel_material.set_uniform(
            CAMERA_TARGET_UNIFORM,
            [camera.target.x, camera.target.y, camera.target.z],
        );
        let (fog_near, fog_far) = Self::calulate_fog_distances(render_size);
        self.voxel_material.set_uniform(FOG_NEAR_UNIFORM, fog_near);
        self.voxel_material.set_uniform(FOG_FAR_UNIFORM, fog_far);
        self.voxel_material
            .set_uniform(LIGHT_LEVEL_UNIFORM, world_time.get_ligth_level());

        gl_use_material(&self.voxel_material);
    }

    fn calulate_fog_distances(render_size: u32) -> (f32, f32) {
        let fog_far = (render_size * AREA_SIZE) as f32;
        let fog_near = fog_far - AREA_SIZE as f32;

        (fog_near, fog_far)
    }
}
