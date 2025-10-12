use std::f32::consts::{PI, TAU};

use bincode::{Decode, Encode};
use macroquad::{
    camera::{Camera3D, set_camera},
    color::Color,
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    rand::gen_range,
    texture::Texture2D,
    window::clear_background,
};

use crate::{
    graphics::{
        mesh_generator::MeshGenerator, mesh_transformer::move_mesh, sky_shader::SkyShader,
        texture_manager::TextureManager,
    },
    service::{
        activity_timer::ActivityTimer, camera_controller::CameraController, world_time::WorldTime,
    },
    utils::{arr_to_vec3, vec3_to_arr},
};

pub const SKY_BRIGHT_COLOR: Color = Color::new(0.75, 0.96, 1.0, 1.0);
pub const SKY_DARK_COLOR: Color = Color::new(0.12, 0.08, 0.36, 1.0);

const DISTANCE_TO_SKY: f32 = 3000.0;
const SUN_AND_MOON_SIZE: f32 = 400.0;

const CLOUDS_SIZE: f32 = 500.0;
const CLOUDS_MIN_X: f32 = 2000.0;
const CLOUDS_MAX_X: f32 = -2000.0;
const CLOUDS_MIN_Z: f32 = -250.0;
const CLOUDS_MAX_Z: f32 = -500.0;
const CLOUDS_SPAWN_DELAY: f32 = 20.0;
const CLOUDS_SPAWN_Y: f32 = -2000.0;
const CLOUDS_DESPAWN_Y: f32 = 2000.0;
const CLOUD_SPEED: f32 = 5.0;

pub struct Sky {
    sky_shader: SkyShader,
    sun_texture: Texture2D,
    moon_texture: Texture2D,
    clouds_manager: CloudsManager,
}
impl Sky {
    pub fn new(texture_manager: &TextureManager) -> Self {
        Self {
            sky_shader: SkyShader::new(),
            sun_texture: texture_manager.get_sun_texture(),
            moon_texture: texture_manager.get_moon_texture(),
            clouds_manager: CloudsManager::new(texture_manager),
        }
    }

    pub fn from_dto(texture_manager: &TextureManager, dto: SkyDTO) -> Self {
        Self {
            sky_shader: SkyShader::new(),
            sun_texture: texture_manager.get_sun_texture(),
            moon_texture: texture_manager.get_moon_texture(),
            clouds_manager: CloudsManager::from_dto(texture_manager, dto.clouds_dto),
        }
    }

    pub fn create_dto(&self) -> SkyDTO {
        SkyDTO {
            clouds_dto: self.clouds_manager.create_dto(),
        }
    }

    pub fn draw_sky(&self, world_time: &WorldTime, camera: &Camera3D) {
        let light_level = world_time.get_light_level();
        let dark_level = 1.0 - light_level;
        let sky_color = Color::new(
            SKY_BRIGHT_COLOR.r * light_level + SKY_DARK_COLOR.r * dark_level,
            SKY_BRIGHT_COLOR.g * light_level + SKY_DARK_COLOR.g * dark_level,
            SKY_BRIGHT_COLOR.b * light_level + SKY_DARK_COLOR.b * dark_level,
            1.0,
        );
        clear_background(sky_color);

        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);

        self.sky_shader.set_sky_material();
        self.draw_sun_and_moon(world_time);
        self.clouds_manager.draw();
    }

    pub fn update(&mut self, delta: f32) {
        self.clouds_manager.update(delta);
    }

    fn draw_sun_and_moon(&self, world_time: &WorldTime) {
        let sun = self.create_sun(world_time);
        let moon = self.create_moon(world_time);

        draw_mesh(&sun);
        draw_mesh(&moon);
    }

    fn create_sun(&self, world_time: &WorldTime) -> Mesh {
        let angle = (world_time.get_delta() * 2.0 + PI) % TAU;
        Mesh {
            texture: Some(self.sun_texture.clone()),
            ..Self::create_sun_or_moon_mesh(angle)
        }
    }

    fn create_moon(&self, world_time: &WorldTime) -> Mesh {
        let angle = world_time.get_delta() * 2.0;
        Mesh {
            texture: Some(self.moon_texture.clone()),
            ..Self::create_sun_or_moon_mesh(angle)
        }
    }

    fn create_sun_or_moon_mesh(angle: f32) -> Mesh {
        let (sin, cos) = angle.sin_cos();
        let mut mesh = MeshGenerator::generate_quad_mesh(SUN_AND_MOON_SIZE);
        for v in &mut mesh.vertices {
            v.position.z -= DISTANCE_TO_SKY;
            let y = v.position.y;
            let z = v.position.z;
            v.position.y = y * cos - z * sin;
            v.position.z = y * sin + z * cos;
        }

        mesh
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct SkyDTO {
    clouds_dto: CloudsDTO,
}

pub struct CloudsManager {
    add_clouds_activity: ActivityTimer,
    clouds_texture: Texture2D,
    cloud_positions: Vec<Vec3>,
}
impl CloudsManager {
    pub fn new(texture_manager: &TextureManager) -> Self {
        let mut clouds = Self {
            clouds_texture: texture_manager.get_clouds_texture(),
            cloud_positions: vec![],
            add_clouds_activity: ActivityTimer::new(0.0, CLOUDS_SPAWN_DELAY),
        };

        clouds.initialise_clouds();

        clouds
    }

    pub fn from_dto(texture_manager: &TextureManager, dto: CloudsDTO) -> Self {
        let cloud_positions = dto.cloud_positions.into_iter().map(arr_to_vec3).collect();

        Self {
            add_clouds_activity: ActivityTimer::new(dto.clouds_spawn_delta, CLOUDS_SPAWN_DELAY),
            clouds_texture: texture_manager.get_clouds_texture(),
            cloud_positions,
        }
    }

    pub fn create_dto(&self) -> CloudsDTO {
        let cloud_positions = self
            .cloud_positions
            .iter()
            .map(|pos| vec3_to_arr(*pos))
            .collect();

        CloudsDTO {
            clouds_spawn_delta: self.add_clouds_activity.get_delta(),
            cloud_positions,
        }
    }

    fn initialise_clouds(&mut self) {
        for _ in 0..50 {
            self.update(CLOUDS_SPAWN_DELAY);
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.remove_distant_clouds();
        self.add_cloud(delta);
        self.move_clouds(delta);
    }

    pub fn draw(&self) {
        for position in &self.cloud_positions {
            let mut cloud = MeshGenerator::generate_quad_mesh(CLOUDS_SIZE);
            move_mesh(&mut cloud, *position);
            cloud.texture = Some(self.clouds_texture.weak_clone());
            draw_mesh(&cloud);
        }
    }

    fn add_cloud(&mut self, delta: f32) {
        if !self.add_clouds_activity.tick(delta) {
            return;
        }

        let x = gen_range(CLOUDS_MIN_X, CLOUDS_MAX_X);
        let y = CLOUDS_SPAWN_Y;
        let z = gen_range(CLOUDS_MIN_Z, CLOUDS_MAX_Z).round();
        self.cloud_positions.push(vec3(x, y, z));
        self.sort_clouds_by_height();
    }

    /// to keep transparency
    fn sort_clouds_by_height(&mut self) {
        self.cloud_positions
            .sort_unstable_by_key(|cloud_position| cloud_position.z as i32);
    }

    fn move_clouds(&mut self, delta: f32) {
        for cloud_position in &mut self.cloud_positions {
            cloud_position.y += delta * CLOUD_SPEED;
        }
    }

    fn remove_distant_clouds(&mut self) {
        self.cloud_positions
            .retain(|cloud_position| cloud_position.y < CLOUDS_DESPAWN_Y);
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CloudsDTO {
    clouds_spawn_delta: f32,
    cloud_positions: Vec<[f32; 3]>,
}
