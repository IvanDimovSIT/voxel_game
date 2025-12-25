use std::sync::Arc;

use bincode::{Decode, Encode};
use macroquad::{
    camera::Camera3D,
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    prelude::info,
    rand::{gen_range, rand},
    texture::Texture2D,
};

use crate::{
    graphics::{
        mesh_generator::MeshGenerator,
        mesh_transformer,
        shader_manager::ShaderManager,
        texture_manager::{PlainTextureId, TextureManager},
    },
    model::{
        area::AREA_SIZE, player_info::PlayerInfo, user_settings::UserSettings, voxel::Voxel,
        world::World,
    },
    service::{
        activity_timer::ActivityTimer,
        sound_manager::{self, SoundManager},
    },
    utils::{arr_to_vec3, vec3_to_arr, vector_to_location},
};

const RAIN_DROP_SIZE: f32 = 0.15;
const MAX_SKY_MODIFIER: f32 = 1.0;
const MIN_SKY_MODIFIER: f32 = 0.6;

const REMOVE_ACTIVITY_COOLDOWN: f32 = 0.1;
const CHANGE_STATE_ACTIVITY_COOLDOWN: f32 = 40.0;

const LIGHTNING_DURATION_S: f32 = 0.8;
const MIN_LIGHTNING_ACTIVITY_COOLDOWN: f32 = 10.0;
const MAX_LIGHTNING_ACTIVITY_COOLDOWN: f32 = 25.0;
const LIGHNING_FLASH_DURATION_S: f32 = 0.3;

fn random_lightning_cooldown() -> f32 {
    gen_range(
        MIN_LIGHTNING_ACTIVITY_COOLDOWN,
        MAX_LIGHTNING_ACTIVITY_COOLDOWN,
    )
}

#[derive(Debug, Clone, Copy)]
pub enum RainLightLevelModifier {
    Multiply(f32),
    Set(f32),
}

#[derive(Debug, Clone, Copy)]
struct RainDrop {
    location: Vec3,
    ground_z: f32,
}
impl From<RainDropDTO> for RainDrop {
    fn from(value: RainDropDTO) -> Self {
        Self {
            location: arr_to_vec3(value.location),
            ground_z: value.ground_z,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Lightning {
    position: Vec3,
    life: f32,
}
impl Lightning {
    fn new(position: Vec3) -> Self {
        Self {
            position,
            life: LIGHTNING_DURATION_S,
        }
    }
}

pub struct RainSystem {
    is_raining: bool,
    rain_drops: Vec<RainDrop>,
    water_texture: Texture2D,
    lightning_texture: Texture2D,
    remove_activity: ActivityTimer,
    change_state_activity: ActivityTimer,
    lightning_activity: ActivityTimer,
    lightnings: Vec<Lightning>,
    sky_modifier: f32,
    /// goes to 0.0 over time, when lightning is added, it increases
    last_lightning_delta: f32,
    shader_manager: Arc<ShaderManager>,
}
impl RainSystem {
    pub fn new(texture_manager: &TextureManager) -> Self {
        Self {
            is_raining: false,
            rain_drops: vec![],
            water_texture: texture_manager.get(Voxel::WaterSource),
            remove_activity: ActivityTimer::new(0.0, REMOVE_ACTIVITY_COOLDOWN),
            sky_modifier: 1.0,
            change_state_activity: ActivityTimer::new(0.0, CHANGE_STATE_ACTIVITY_COOLDOWN),
            lightning_activity: ActivityTimer::new(0.0, random_lightning_cooldown()),
            lightnings: vec![],
            last_lightning_delta: 0.0,
            lightning_texture: texture_manager.get_plain_texture(PlainTextureId::Lightning),
            shader_manager: ShaderManager::instance(),
        }
    }

    pub fn from_dto(dto: RainSystemDTO, texture_manager: &TextureManager) -> Self {
        let rain_drops = dto
            .rain_drops
            .into_iter()
            .map(|rain_dto| rain_dto.into())
            .collect();

        Self {
            is_raining: dto.is_raining,
            rain_drops,
            water_texture: texture_manager.get(Voxel::WaterSource),
            remove_activity: ActivityTimer::new(dto.remove_delta, REMOVE_ACTIVITY_COOLDOWN),
            change_state_activity: ActivityTimer::new(
                dto.change_state_delta,
                CHANGE_STATE_ACTIVITY_COOLDOWN,
            ),
            sky_modifier: dto.sky_modifier,
            lightning_activity: dto.lightning_activity,
            lightnings: vec![],
            last_lightning_delta: 0.0,
            lightning_texture: texture_manager.get_plain_texture(PlainTextureId::Lightning),
            shader_manager: ShaderManager::instance(),
        }
    }

    pub fn create_dto(&self) -> RainSystemDTO {
        let rain_drops = self
            .rain_drops
            .iter()
            .map(|rain_drop| (*rain_drop).into())
            .collect();

        RainSystemDTO {
            is_raining: self.is_raining,
            rain_drops,
            remove_delta: self.remove_activity.get_delta(),
            change_state_delta: self.change_state_activity.get_delta(),
            sky_modifier: self.sky_modifier,
            lightning_activity: self.lightning_activity,
        }
    }

    pub fn update(
        &mut self,
        delta: f32,
        player_info: &PlayerInfo,
        world: &mut World,
        user_settings: &UserSettings,
        sound_manager: &SoundManager,
    ) {
        if self.change_state_activity.tick(delta) {
            self.update_change_raining_state();
        }

        if self.is_raining {
            let mut start_position = player_info.camera_controller.get_position();
            start_position.z -= 20.0;
            self.add_rain(delta, start_position, world);
        }
        self.update_sky_modifier(delta);

        self.simulate(delta);
        if self.remove_activity.tick(delta) {
            self.remove_fallen();
        }

        self.update_lightning(delta, player_info, world, user_settings, sound_manager);
    }

    /// draws rain drops as quads facing at the camera
    pub fn draw_rain(&self, camera: &Camera3D) {
        let camera_position = camera.position;
        let look = (camera.target - camera_position).normalize_or_zero();

        self.rain_drops
            .iter()
            .filter(|r| Self::cull_visible(camera_position, look, r.location))
            .for_each(|r| self.draw_mesh_for_rain_drop(r, camera_position));
    }

    /// should be used with the sky shader
    pub fn draw_lightning(&self, camera: &Camera3D) {
        if self.lightnings.is_empty() {
            return;
        }

        self.shader_manager.flat_shader.set_flat_material(camera);
        let camera_position = camera.position;
        for l in &self.lightnings {
            let mesh = self.create_lightning_mesh(l.position, camera_position);
            draw_mesh(&mesh);
        }
    }

    pub fn get_light_level_modifier(&self) -> RainLightLevelModifier {
        if self.last_lightning_delta > 0.0 {
            RainLightLevelModifier::Set(1.0)
        } else {
            RainLightLevelModifier::Multiply(self.sky_modifier)
        }
    }

    /// updates lightning - adds or removes
    fn update_lightning(
        &mut self,
        delta: f32,
        player_info: &PlayerInfo,
        world: &mut World,
        user_settings: &UserSettings,
        sound_manager: &SoundManager,
    ) {
        self.last_lightning_delta = (self.last_lightning_delta - delta).max(0.0);
        for lightning in &mut self.lightnings {
            lightning.life -= delta;
        }

        self.lightnings.retain(|l| l.life > 0.0);

        let should_add_lightning = self.is_raining
            && self
                .lightning_activity
                .tick_change_cooldown(delta, random_lightning_cooldown);
        if should_add_lightning {
            self.add_lightning(player_info, world, user_settings, sound_manager);
        }
    }

    /// creates a ligtning at a random position around the player
    fn add_lightning(
        &mut self,
        player_info: &PlayerInfo,
        world: &mut World,
        user_settings: &UserSettings,
        sound_manager: &SoundManager,
    ) {
        let max_add_distance = ((user_settings.get_render_distance() - 1) * AREA_SIZE) as f32;

        let x_offset = gen_range(-max_add_distance, max_add_distance);
        let y_offset = gen_range(-max_add_distance, max_add_distance);
        let sample_location = vector_to_location(
            player_info.camera_controller.get_position() + vec3(x_offset, y_offset, 0.0),
        );

        let ground_z = world.get_non_empty_height_without_loading(sample_location);
        let lightning_position = vec3(
            sample_location.x as f32,
            sample_location.y as f32,
            ground_z as f32,
        );

        let lightning = Lightning::new(lightning_position);
        info!("Lightning at {}", lightning_position);
        sound_manager.play_sound(sound_manager::SoundId::Thunder, user_settings);

        self.lightnings.push(lightning);
        self.last_lightning_delta = LIGHNING_FLASH_DURATION_S;
    }

    fn create_lightning_mesh(&self, lightning_position: Vec3, camera_position: Vec3) -> Mesh {
        let mesh = MeshGenerator::generate_lightning_mesh(lightning_position, camera_position);

        Mesh {
            texture: Some(self.lightning_texture.weak_clone()),
            ..mesh
        }
    }

    #[inline(always)]
    fn cull_visible(camera_position: Vec3, look: Vec3, mesh_position: Vec3) -> bool {
        const VISIBLE_THRESHOLD: f32 = 0.6;
        let look_towards_rain = (mesh_position - camera_position).normalize_or_zero();

        look.dot(look_towards_rain) >= VISIBLE_THRESHOLD
    }

    fn update_change_raining_state(&mut self) {
        const START_RAINING_CHANCE: u32 = 20;
        const STOP_RAINING_CHANCE: u32 = 50;
        let random_percent = rand() % 100;

        if self.is_raining && random_percent < STOP_RAINING_CHANCE {
            info!(
                "Stopping rain, rand: {}, MAX:{}",
                random_percent, STOP_RAINING_CHANCE
            );
            self.is_raining = false;
        } else if !self.is_raining && random_percent < START_RAINING_CHANCE {
            info!(
                "Starting rain, rand: {}, MAX:{}",
                random_percent, START_RAINING_CHANCE
            );
            self.is_raining = true;
        } else {
            info!("No change in rain, rand: {}", random_percent);
        }
    }

    fn update_sky_modifier(&mut self, delta: f32) {
        const CHANGE_MODIFIER_SPEED: f32 = 0.1;
        if self.is_raining {
            self.sky_modifier =
                (self.sky_modifier - CHANGE_MODIFIER_SPEED * delta).max(MIN_SKY_MODIFIER);
        } else {
            self.sky_modifier =
                (self.sky_modifier + CHANGE_MODIFIER_SPEED * delta).min(MAX_SKY_MODIFIER);
        }
    }

    fn draw_mesh_for_rain_drop(&self, rain_drop: &RainDrop, facing: Vec3) {
        let mut mesh = MeshGenerator::generate_quad_mesh(RAIN_DROP_SIZE);
        mesh_transformer::move_mesh(&mut mesh, rain_drop.location);
        mesh_transformer::rotate_mesh_towards(
            &mut mesh,
            vec3(0.0, 0.0, 1.0),
            rain_drop.location,
            facing,
        );

        mesh.texture = Some(self.water_texture.weak_clone());

        draw_mesh(&mesh);
    }

    fn remove_fallen(&mut self) {
        self.rain_drops
            .retain(|rain_drop| rain_drop.location.z < rain_drop.ground_z);
    }

    fn add_rain(&mut self, delta: f32, start_position: Vec3, world: &mut World) {
        const MAX_DISTANCE: f32 = 32.0;
        const SPAWN_COUNT_PER_S: f32 = 1100.0;

        let spawn_count = (SPAWN_COUNT_PER_S * delta) as u32;

        for _ in 0..spawn_count {
            let x_offset = gen_range(-MAX_DISTANCE, MAX_DISTANCE);
            let y_offset = gen_range(-MAX_DISTANCE, MAX_DISTANCE);
            let z_offset = gen_range(-2.0, 2.0);
            let drop_location = vec3(x_offset, y_offset, z_offset) + start_position;
            let ground_z =
                world.get_non_empty_height_without_loading(vector_to_location(drop_location));

            let rain_drop = RainDrop {
                location: drop_location,
                ground_z: ground_z as f32 - Voxel::HALF_SIZE,
            };
            self.rain_drops.push(rain_drop);
        }
    }

    fn simulate(&mut self, delta: f32) {
        const FALL_SPEED: f32 = 12.0;
        for drop in &mut self.rain_drops {
            drop.location.z += FALL_SPEED * delta;
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
struct RainDropDTO {
    location: [f32; 3],
    ground_z: f32,
}
impl From<RainDrop> for RainDropDTO {
    fn from(value: RainDrop) -> Self {
        Self {
            location: vec3_to_arr(value.location),
            ground_z: value.ground_z,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct RainSystemDTO {
    is_raining: bool,
    rain_drops: Vec<RainDropDTO>,
    remove_delta: f32,
    change_state_delta: f32,
    sky_modifier: f32,
    lightning_activity: ActivityTimer,
}
