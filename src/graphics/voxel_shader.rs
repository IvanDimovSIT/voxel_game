use macroquad::{
    camera::Camera3D,
    math::{Vec3, Vec4Swizzles, vec3},
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::{
        Comparison, Material, MaterialParams, PipelineParams, ShaderSource, UniformDesc,
        UniformType, gl_use_material, load_material,
    },
};

use crate::{
    graphics::sky::{SKY_BRIGHT_COLOR, SKY_DARK_COLOR},
    model::{
        area::AREA_SIZE,
        location::{InternalLocation, Location},
    },
    service::world_time::WorldTime,
};

const SHADOW_CHANGE_STEEPNESS: f32 = 10.0;
const MAX_LIGHTS: usize = 32;

const VOXEL_VERTEX_SHADER: &str = include_str!("../../resources/shaders/voxel_vertex.glsl");
const VOXEL_FRAGMENT_SHADER: &str = include_str!("../../resources/shaders/voxel_fragment.glsl");

const CAMERA_POSITION_UNIFORM: &str = "cameraPos";
const CAMERA_TARGET_UNIFORM: &str = "cameraTarget";
const FOG_NEAR_UNIFORM: &str = "fogNear";
const FOG_FAR_UNIFORM: &str = "fogFar";
const LIGHT_LEVEL_UNIFORM: &str = "lightLevel";
const FOG_BASE_COLOR_LIGHT_UNIFORM: &str = "fogBaseColorLight";
const FOG_BASE_COLOR_DARK_UNIFORM: &str = "fogBaseColorDark";
const SHADOW_AMOUNT_UNIFORM: &str = "shadowAmount";
const LIGHTS_COUNT_UNIFORM: &str = "lightsCount";
const LIGHTS_UNIFORM: &str = "lights";

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
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            ..Default::default()
        };

        let camera_uniform = UniformDesc::new(CAMERA_POSITION_UNIFORM, UniformType::Float3);
        let look_uniform = UniformDesc::new(CAMERA_TARGET_UNIFORM, UniformType::Float3);
        let fog_near_uniform = UniformDesc::new(FOG_NEAR_UNIFORM, UniformType::Float1);
        let fog_far_uniform = UniformDesc::new(FOG_FAR_UNIFORM, UniformType::Float1);
        let light_level_uniform = UniformDesc::new(LIGHT_LEVEL_UNIFORM, UniformType::Float1);
        let fog_light_color_uniform =
            UniformDesc::new(FOG_BASE_COLOR_LIGHT_UNIFORM, UniformType::Float3);
        let fog_dark_color_uniform =
            UniformDesc::new(FOG_BASE_COLOR_DARK_UNIFORM, UniformType::Float3);
        let average_max_height_uniform =
            UniformDesc::new(SHADOW_AMOUNT_UNIFORM, UniformType::Float1);
        let lights_count_uniform = UniformDesc::new(LIGHTS_COUNT_UNIFORM, UniformType::Int1);
        let lights_uniform =
            UniformDesc::new(LIGHTS_UNIFORM, UniformType::Float3).array(MAX_LIGHTS);

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
                    fog_light_color_uniform,
                    fog_dark_color_uniform,
                    average_max_height_uniform,
                    lights_count_uniform,
                    lights_uniform,
                ],
                ..Default::default()
            },
        )
        .expect("Error initialising voxel shaders");

        Self { voxel_material }
    }

    /// sets the current OpenGL shader to render the world voxels
    pub fn set_voxel_material(
        &self,
        camera: &Camera3D,
        render_size: u32,
        world_time: &WorldTime,
        average_max_height: Option<f32>,
        lights: &[InternalLocation],
    ) {
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
        self.voxel_material.set_uniform(
            FOG_BASE_COLOR_LIGHT_UNIFORM,
            SKY_BRIGHT_COLOR.to_vec().xyz(),
        );
        self.voxel_material
            .set_uniform(FOG_BASE_COLOR_DARK_UNIFORM, SKY_DARK_COLOR.to_vec().xyz());
        let shadow_amount = average_max_height
            .map(|max_z| Self::calculate_shadow_amount(max_z, camera))
            .unwrap_or(0.0);
        self.voxel_material
            .set_uniform(SHADOW_AMOUNT_UNIFORM, shadow_amount);
        self.set_lights(lights, camera);

        gl_use_material(&self.voxel_material);
    }

    fn set_lights(&self, lights: &[InternalLocation], camera: &Camera3D) {
        let mut lights_array: [Vec3; MAX_LIGHTS] = [Vec3::ZERO; MAX_LIGHTS];
        let lights_count = lights.len().min(MAX_LIGHTS);
        let lights_iter = lights
            .iter()
            .take(MAX_LIGHTS)
            .map(|internal_location| {
                let location: Location = (*internal_location).into();
                vec3(
                    location.x as f32 - camera.position.x,
                    location.y as f32 - camera.position.y,
                    location.z as f32 - camera.position.z,
                )
            })
            .enumerate();

        for (i, light_position) in lights_iter {
            lights_array[i] = light_position;
        }

        self.voxel_material
            .set_uniform_array(LIGHTS_UNIFORM, &lights_array);
        self.voxel_material
            .set_uniform(LIGHTS_COUNT_UNIFORM, lights_count as i32);
    }

    fn calculate_shadow_amount(average_max_height: f32, camera: &Camera3D) -> f32 {
        let player_z = camera.position.z - 1.0;
        if average_max_height >= player_z {
            0.0
        } else {
            ((player_z - average_max_height) / SHADOW_CHANGE_STEEPNESS).clamp(0.0, 0.7)
        }
    }

    fn calulate_fog_distances(render_size: u32) -> (f32, f32) {
        let fog_far = (render_size * AREA_SIZE) as f32;
        let fog_near = fog_far - AREA_SIZE as f32;

        (fog_near, fog_far)
    }
}
