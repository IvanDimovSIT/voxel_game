use std::f32::consts::PI;

use macroquad::{
    camera::Camera3D,
    input::{mouse_position, set_cursor_grab, show_mouse},
    math::{Vec2, Vec3, vec3},
};

use crate::{model::location::Location, utils::vector_to_location};

const LOOK_SPEED: f32 = 0.1;

#[derive(Debug)]
pub struct CameraController {
    is_focused: bool,
    pub yaw: f32,
    pub pitch: f32,
    front: Vec3,
    right: Vec3,
    up: Vec3,
    world_up: Vec3,
    last_mouse_position: Vec2,
    position: Vec3,
}
impl CameraController {
    pub fn new(position: Vec3) -> Self {
        let world_up = vec3(0.0, 0.0, -1.0);
        let yaw: f32 = PI / 3.0;
        let pitch: f32 = 0.0;
        let front = vec3(
            yaw.cos() * pitch.cos(),
            yaw.sin() * pitch.cos(),
            pitch.sin(),
        )
        .normalize();
        let right = front.cross(world_up).normalize();
        let up = right.cross(front).normalize();
        let last_mouse_position: Vec2 = mouse_position().into();

        Self {
            is_focused: false,
            yaw,
            pitch,
            front,
            right,
            up,
            world_up,
            last_mouse_position,
            position,
        }
    }

    /// sets window focus
    pub fn set_focus(&mut self, is_focused: bool) {
        self.is_focused = is_focused;
        set_cursor_grab(is_focused);
        show_mouse(!is_focused);
    }

    pub fn update_look(&mut self, delta: f32) {
        let delta = delta.min(0.03);
        let mouse_position: Vec2 = mouse_position().into();
        let mouse_delta = mouse_position - self.last_mouse_position;
        self.last_mouse_position = mouse_position;

        if !self.is_focused {
            return;
        }

        self.yaw += mouse_delta.x * delta * LOOK_SPEED;
        self.pitch += mouse_delta.y * delta * LOOK_SPEED;

        self.pitch = if self.pitch > 1.5 { 1.5 } else { self.pitch };
        self.pitch = if self.pitch < -1.5 { -1.5 } else { self.pitch };

        self.front = vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
        .normalize();

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }

    pub fn get_forward_displacement(&self, speed: f32, delta: f32) -> Vec3 {
        Self::ignore_z(self.front) * speed * delta
    }

    pub fn get_right_displacement(&self, speed: f32, delta: f32) -> Vec3 {
        Self::ignore_z(self.right) * speed * delta
    }

    pub fn move_forward(&mut self, speed: f32, delta: f32) {
        self.position += self.get_forward_displacement(speed, delta);
    }

    pub fn move_right(&mut self, speed: f32, delta: f32) {
        self.position += self.get_right_displacement(speed, delta);
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }

    pub fn get_bottom_position(&self) -> Vec3 {
        self.position + vec3(0.0, 0.0, 1.5)
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn create_camera(&self) -> Camera3D {
        Camera3D {
            position: self.position,
            up: self.up,
            target: self.position + self.front,
            ..Default::default()
        }
    }

    pub fn get_camera_voxel_location(&self) -> Location {
        vector_to_location(self.position)
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// Normalise the camera position to (0,0,0)
    pub fn normalize_camera_3d(camera: &Camera3D) -> Camera3D {
        Camera3D {
            position: Vec3::ZERO,
            target: camera.target - camera.position,
            up: camera.up,
            fovy: camera.fovy,
            aspect: camera.aspect,
            projection: camera.projection,
            render_target: camera.render_target.clone(),
            viewport: camera.viewport,
            z_near: camera.z_near,
            z_far: camera.z_far,
        }
    }

    fn ignore_z(vec: Vec3) -> Vec3 {
        vec3(vec.x, vec.y, 0.0).normalize_or_zero()
    }
}
