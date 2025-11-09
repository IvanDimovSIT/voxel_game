use bincode::{Decode, Encode};
use macroquad::{
    camera::Camera3D,
    math::{Vec3, vec3},
    models::draw_mesh,
    prelude::info,
    rand::{gen_range, rand},
    texture::Texture2D,
};

use crate::{
    graphics::{mesh_generator::MeshGenerator, mesh_transformer, texture_manager::TextureManager},
    model::{player_info::PlayerInfo, voxel::Voxel, world::World},
    service::activity_timer::ActivityTimer,
    utils::{arr_to_vec3, vec3_to_arr, vector_to_location},
};

const RAIN_DROP_SIZE: f32 = 0.15;
const MAX_SKY_MODIFIER: f32 = 1.0;
const MIN_SKY_MODIFIER: f32 = 0.6;

const REMOVE_ACTIVITY_COOLDOWN: f32 = 0.1;
const CHANGE_STATE_ACTIVITY_COOLDOWN: f32 = 40.0;

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

pub struct RainSystem {
    is_raining: bool,
    rain_drops: Vec<RainDrop>,
    water_texture: Texture2D,
    remove_activity: ActivityTimer,
    change_state_activity: ActivityTimer,
    sky_modifier: f32,
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
        }
    }

    pub fn update(&mut self, delta: f32, player_info: &PlayerInfo, world: &mut World) {
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
    }

    /// draws rain drops as quads facing at the camera
    pub fn draw(&self, camera: &Camera3D) {
        let camera_position = camera.position;
        let look = (camera.target - camera_position).normalize_or_zero();

        self.rain_drops
            .iter()
            .filter(|r| Self::cull_visible(camera_position, look, r.location))
            .for_each(|r| self.draw_mesh_for_rain_drop(r, camera_position));
    }

    pub fn get_light_level_modifier(&self) -> f32 {
        self.sky_modifier
    }

    #[inline(always)]
    fn cull_visible(camera_position: Vec3, look: Vec3, rain_location: Vec3) -> bool {
        const VISIBLE_THRESHOLD: f32 = 0.6;
        let look_towards_rain = (rain_location - camera_position).normalize_or_zero();

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
            let ground_z = world.get_non_empty_height(vector_to_location(drop_location));

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
}
