use std::f32::consts::TAU;

use bincode::{Decode, Encode};
use macroquad::{
    math::{Vec3, vec3},
    models::Mesh,
    rand::{gen_range, rand},
};

use crate::{
    graphics::mesh_manager::{MeshId, MeshManager},
    model::{area::AREA_HEIGHT, voxel::Voxel, world::World},
    service::{
        activity_timer::ActivityTimer,
        creatures::creature_manager::{Creature, CreatureDTO, CreatureId, CreatureManager},
        physics::player_physics::{GRAVITY, MAX_FALL_SPEED},
    },
    utils::{arr_to_vec3, vec3_to_arr, vector_to_location},
};

const SIZE: Vec3 = vec3(0.8, 0.8, 0.8);
const SPEED: f32 = 3.0;
const JUMP: f32 = -12.0;
const TURN_SPEED: f32 = 2.2;
const WAIT_ACTIVITY_MAX: f32 = 3.0;
const MOVE_ACTIVITY_MAX: f32 = 14.0;
const TURN_ACTIVITY: f32 = 1.5;
const MIN_ACTIVITY: f32 = 0.5;

const SWIM_SPEED: f32 = -30.0;
const MAX_SWIM: f32 = -8.0;

const FORWAD_DIRECTION: Vec3 = vec3(0.0, 1.0, 0.0);

#[derive(Debug, Clone, Copy, Encode, Decode)]
enum Activity {
    Idle,
    Move,
    Turn(bool),
}

pub struct BunnyCreature {
    activity_timer: ActivityTimer,
    position: Vec3,
    velocity: f32,
    activity: Activity,
    direction: Vec3,
    rotation: f32,
    mesh: Mesh,
}
impl BunnyCreature {
    /// creates a new bunny creature at position with a random rotation
    pub fn new(position: Vec3, mesh_manager: &MeshManager) -> Self {
        let mesh = mesh_manager.get_at(MeshId::Bunny, position);
        let random_rotation = gen_range(0.0, TAU);
        let mut bunny = Self {
            position,
            velocity: 0.0,
            mesh,
            activity_timer: ActivityTimer::new(0.0, gen_range(MIN_ACTIVITY, WAIT_ACTIVITY_MAX)),
            activity: Activity::Idle,
            direction: FORWAD_DIRECTION,
            rotation: random_rotation,
        };

        MeshManager::rotate_around_z_with_direction(
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

    fn handle_move(&mut self, delta: f32, world: &mut World, on_ground: bool) {
        let move_distance = delta * SPEED;

        if on_ground {
            self.velocity = JUMP;
        }

        let displacement = self.direction * move_distance;
        self.position += displacement;
        if CreatureManager::collides(self, world) {
            self.position -= displacement;
        }
    }

    fn handle_turn(&mut self, delta: f32, clockwise: bool) {
        let turn_amount = if clockwise {
            TAU - delta * TURN_SPEED
        } else {
            delta * TURN_SPEED
        };

        self.rotation += turn_amount;
        if self.rotation > TAU {
            self.rotation -= TAU;
        } else if self.rotation < 0.0 {
            self.rotation += TAU;
        }

        MeshManager::rotate_around_z_with_direction(
            &mut self.mesh,
            &mut self.direction,
            self.position,
            turn_amount,
        );
    }

    fn swim_if_in_water(&mut self, delta: f32, world: &mut World) {
        let voxel = world.get(vector_to_location(self.position));
        if !Voxel::WATER.contains(&voxel) {
            return;
        }

        self.velocity += delta * SWIM_SPEED;
        self.velocity = self.velocity.max(MAX_SWIM);
    }
}
impl Creature for BunnyCreature {
    fn update(&mut self, delta: f32, world: &mut World) {
        debug_assert!(self.position.z >= 0.0);
        debug_assert!(self.position.z < AREA_HEIGHT as f32);
        let old_position = self.position;
        if self.activity_timer.tick(delta) {
            (self.activity, self.activity_timer) = match self.activity {
                Activity::Idle => (
                    Activity::Turn(rand() % 2 == 0),
                    ActivityTimer::new(MIN_ACTIVITY, TURN_ACTIVITY),
                ),
                Activity::Move => (
                    Activity::Idle,
                    ActivityTimer::new(0.0, gen_range(MIN_ACTIVITY, WAIT_ACTIVITY_MAX)),
                ),
                Activity::Turn(_) => (
                    Activity::Move,
                    ActivityTimer::new(0.0, gen_range(MIN_ACTIVITY, MOVE_ACTIVITY_MAX)),
                ),
            }
        }
        let on_ground = self.handle_gravity(delta, world);
        self.swim_if_in_water(delta, world);

        match self.activity {
            Activity::Idle => {}
            Activity::Move => {
                self.handle_move(delta, world, on_ground);
            }
            Activity::Turn(clockwise) => {
                self.handle_turn(delta, clockwise);
            }
        }

        let delta_position = self.position - old_position;
        if delta_position != Vec3::ZERO {
            MeshManager::move_mesh(&mut self.mesh, delta_position);
        }
    }

    fn get_mesh_with_index(&self) -> (&Mesh, usize) {
        (&self.mesh, MeshId::Bunny.index())
    }

    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_size(&self) -> Vec3 {
        SIZE
    }

    fn create_dto(&self) -> Option<CreatureDTO> {
        let dto = BunnyDTO {
            activity_timer: self.activity_timer,
            position: vec3_to_arr(self.position),
            velocity: self.velocity,
            activity: self.activity,
            rotation: self.rotation,
        };

        CreatureManager::encode_creature_dto(&dto, CreatureId::Bunny)
    }

    fn from_dto(
        creature_dto: CreatureDTO,
        mesh_manager: &MeshManager,
    ) -> Option<Box<dyn Creature>> {
        let bunny_dto: BunnyDTO =
            CreatureManager::decode_creature_dto(creature_dto, CreatureId::Bunny)?;

        let position = arr_to_vec3(bunny_dto.position);
        let mut mesh = mesh_manager.get_at(MeshId::Bunny, position);
        let mut direction = FORWAD_DIRECTION;
        MeshManager::rotate_around_z_with_direction(
            &mut mesh,
            &mut direction,
            position,
            bunny_dto.rotation,
        );

        Some(Box::new(Self {
            activity_timer: bunny_dto.activity_timer,
            position,
            velocity: bunny_dto.velocity,
            activity: bunny_dto.activity,
            direction,
            mesh,
            rotation: bunny_dto.rotation,
        }))
    }
}

#[derive(Debug, Encode, Decode)]
struct BunnyDTO {
    activity_timer: ActivityTimer,
    rotation: f32,
    position: [f32; 3],
    velocity: f32,
    activity: Activity,
}
