use macroquad::{
    camera::Camera3D,
    math::{Vec3, vec3},
    models::draw_mesh,
    rand::gen_range,
    texture::Texture2D,
};

use crate::{
    graphics::{mesh_generator::MeshGenerator, mesh_transformer, texture_manager::TextureManager},
    model::{player_info::PlayerInfo, voxel::Voxel, world::World},
    service::activity_timer::ActivityTimer,
    utils::vector_to_location,
};

const RAIN_DROP_SIZE: f32 = 0.15;
const MAX_SKY_MODIFIER: f32 = 1.0;
const MIN_SKY_MODIFIER: f32 = 0.6;

struct RainDrop {
    location: Vec3,
    ground_z: f32,
}

pub struct RainSystem {
    is_raining: bool,
    rain_drops: Vec<RainDrop>,
    water_texture: Texture2D,
    remove_activity: ActivityTimer,
    sky_modifier: f32,
}
impl RainSystem {
    pub fn new(texture_manager: &TextureManager) -> Self {
        Self {
            is_raining: true,
            rain_drops: vec![],
            water_texture: texture_manager.get(Voxel::WaterSource),
            remove_activity: ActivityTimer::new(0.0, 0.1),
            sky_modifier: 1.0,
        }
    }

    pub fn update(&mut self, delta: f32, player_info: &PlayerInfo, world: &mut World) {
        if self.is_raining {
            let mut start_position = player_info.camera_controller.get_position();
            start_position.z -= 20.0;
            self.add_rain(start_position, world);
        }
        self.update_sky_modifier(delta);

        self.simulate(delta);
        if self.remove_activity.tick(delta) {
            self.remove_fallen();
        }
    }

    /// draws rain drops as quads facing at the camera
    pub fn draw(&self, camera: &Camera3D) {
        if !self.is_raining {
            return;
        }

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

    fn add_rain(&mut self, start_position: Vec3, world: &mut World) {
        const MAX_DISTANCE: f32 = 30.0;
        const SPAWN_COUNT: usize = 20;

        for _ in 0..SPAWN_COUNT {
            let x_offset = gen_range(-MAX_DISTANCE, MAX_DISTANCE);
            let y_offset = gen_range(-MAX_DISTANCE, MAX_DISTANCE);
            let drop_location = vec3(x_offset, y_offset, 0.0) + start_position;
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
