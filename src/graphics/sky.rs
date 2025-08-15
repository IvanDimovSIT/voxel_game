use std::f32::consts::{PI, TAU};

use macroquad::{
    camera::{Camera3D, set_camera},
    color::Color,
    models::{Mesh, draw_mesh},
    texture::Texture2D,
    window::clear_background,
};

use crate::{
    graphics::{
        mesh_generator::MeshGenerator, sky_shader::SkyShader, texture_manager::TextureManager,
    },
    service::{camera_controller::CameraController, world_time::WorldTime},
};

pub const SKY_BRIGHT_COLOR: Color = Color::new(0.92, 0.94, 0.61, 1.0);
pub const SKY_DARK_COLOR: Color = Color::new(0.12, 0.08, 0.36, 1.0);
pub const DISTANCE_TO_SKY: f32 = 400.0;
pub const SUN_AND_MOON_SIZE: f32 = 50.0;

pub struct Sky {
    sky_shader: SkyShader,
    sun_texture: Texture2D,
    moon_texture: Texture2D,
}
impl Sky {
    pub fn new(texture_manager: &TextureManager) -> Self {
        Self {
            sky_shader: SkyShader::new(),
            sun_texture: texture_manager.get_sun_texture(),
            moon_texture: texture_manager.get_moon_texture(),
        }
    }

    pub fn draw_sky(&self, world_time: &WorldTime, camera: &Camera3D) {
        let light_level = world_time.get_ligth_level();
        let dark_level = 1.0 - light_level;
        let sky_color = Color::new(
            SKY_BRIGHT_COLOR.r * light_level + SKY_DARK_COLOR.r * dark_level,
            SKY_BRIGHT_COLOR.g * light_level + SKY_DARK_COLOR.g * dark_level,
            SKY_BRIGHT_COLOR.b * light_level + SKY_DARK_COLOR.b * dark_level,
            1.0,
        );
        clear_background(sky_color);

        let sun = self.create_sun(world_time);
        let moon = self.create_moon(world_time);

        let normalised_camera = CameraController::normalize_camera_3d(camera);
        set_camera(&normalised_camera);
        self.sky_shader.set_sky_material();
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
