use std::f32::consts::TAU;

use macroquad::{
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    rand::gen_range,
};

use crate::{
    graphics::mesh_manager::MeshManager,
    model::{area::AREA_HEIGHT, world::World},
    service::{
        activity_timer::ActivityTimer,
        creatures::creature_manager::{Creature, CreatureManager},
        physics::player_physics::{GRAVITY, MAX_FALL_SPEED},
    },
};

const SIZE: Vec3 = vec3(0.8, 0.8, 0.8);
const SPEED: f32 = 2.0;
const JUMP: f32 = -15.0;
const ACTIVITY: f32 = 7.0;

enum Activity {
    Idle,
    Move(f32),
}

pub struct BunnyCreature {
    activity_timer: ActivityTimer,
    position: Vec3,
    velocity: f32,
    activity: Activity,
    direction: Vec3,
    mesh: Mesh,
}
impl BunnyCreature {
    /// creates a new bunny creature at position with a random rotation
    pub fn new(position: Vec3, mesh: Mesh) -> Self {
        let mut bunny = Self {
            position,
            velocity: 0.0,
            mesh,
            activity_timer: ActivityTimer::new(0.0, ACTIVITY),
            activity: Activity::Idle,
            direction: vec3(0.0, 1.0, 0.0),
        };

        let random_rotation = gen_range(0.0, TAU);
        MeshManager::rotate_around_z(
            &mut bunny.mesh,
            &mut bunny.direction,
            bunny.position,
            random_rotation,
        );

        bunny
    }

    /// returns true if on the ground
    fn handle_gravity(&mut self, delta: f32, world: &mut World) -> bool {
        self.velocity += delta * GRAVITY;
        self.velocity = self.velocity.min(MAX_FALL_SPEED);
        self.position.z += self.velocity * delta;
        let (new_z, is_on_ground) = CreatureManager::collides_with_ground(self, world);

        if new_z > self.position.z || is_on_ground {
            self.velocity = 0.0;
        }
        self.position.z = new_z;

        is_on_ground
    }

    /// returns new move distance
    fn handle_move(&mut self, delta: f32, world: &mut World, to_move: f32, on_ground: bool) -> f32 {
        let move_distance = (delta * SPEED).min(to_move);
        if move_distance <= f32::EPSILON {
            return 0.0;
        }

        if on_ground {
            self.velocity = JUMP;
        }

        let displacement = self.direction * move_distance;
        self.position += displacement;
        if CreatureManager::collides(self, world) {
            self.position -= displacement;

            return to_move;
        }

        move_distance
    }
}
impl Creature for BunnyCreature {
    fn update(&mut self, delta: f32, world: &mut World) {
        debug_assert!(self.position.z >= 0.0);
        debug_assert!(self.position.z < AREA_HEIGHT as f32);
        let old_position = self.position;
        if self.activity_timer.tick(delta) {
            self.activity = match self.activity {
                Activity::Idle => Activity::Move(gen_range(1.0, 10.0)),
                Activity::Move(_) => Activity::Idle,
            }
        }
        let on_ground = self.handle_gravity(delta, world);

        match self.activity {
            Activity::Idle => {}
            Activity::Move(amount) => {
                let new_amount = self.handle_move(delta, world, amount, on_ground);
                self.activity = Activity::Move(new_amount);
            }
        }

        let delta_position = self.position - old_position;
        if delta_position != Vec3::ZERO {
            MeshManager::move_mesh(&mut self.mesh, delta_position);
        }
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
