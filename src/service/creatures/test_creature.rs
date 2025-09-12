use macroquad::{math::{vec3, Vec3}, models::{draw_mesh, Mesh}, prelude::info};

use crate::{model::world::World, service::{activity_timer::ActivityTimer, creatures::creature_manager::Creature}};

const SIZE: Vec3 = vec3(1.0, 1.0, 1.0);

pub struct TestCreature {
    activity_timer: ActivityTimer,
    position: Vec3,
    velocity: f32,
    rotation: f32,
    mesh: Mesh
}
impl TestCreature {
    pub fn new(position: Vec3, mesh: Mesh) -> Self {
        Self { 
            position, 
            velocity: 0.0, 
            mesh,
            activity_timer: ActivityTimer::new(0.0, 5.0),
            rotation: 0.0,
        }
    }
}
impl Creature for TestCreature {
    fn update(&mut self, delta: f32, world: &mut World) {
        info!("Updated");
    }

    fn draw(&self) {
        draw_mesh(&self.mesh);
    }

    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_size(&self) -> Vec3 {
        SIZE
    }
}