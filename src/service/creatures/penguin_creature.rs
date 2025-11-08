use std::f32::consts::TAU;

use bincode::{Decode, Encode};
use macroquad::{
    math::{Vec3, vec3},
    models::Mesh,
    rand::{gen_range, rand},
};

use crate::{
    graphics::{
        mesh_manager::{MeshId, MeshManager},
        mesh_transformer::{move_mesh, rotate_around_z_with_direction},
    },
    model::{player_info::PlayerInfo, voxel::Voxel, world::World},
    service::{
        activity_timer::ActivityTimer,
        creatures::{
            creature::{
                Creature, collides, collides_with_ground, collides_with_player,
                perform_static_collisions, push_away_from,
            },
            creature_manager::{CreatureDTO, CreatureId, CreatureManager},
        },
        physics::player_physics::{GRAVITY, MAX_FALL_SPEED},
    },
    utils::{arr_to_vec3, vec3_to_arr, vector_to_location},
};

const SIZE: Vec3 = vec3(0.8, 0.8, 1.0);
const FORWAD_DIRECTION: Vec3 = vec3(0.0, 1.0, 0.0);
const TURN_SPEED: f32 = 0.4;
const IDLE_ACTIVITY_TIME: f32 = 2.0;
const WALK_ACTIVITY_TIME: f32 = 8.0;
const TURN_ACTIVITY_TIME: f32 = 4.0;
const SPEED: f32 = 1.1;
const SWIM_SPEED: f32 = -45.0;
const MAX_SWIM: f32 = -4.0;
const JUMP: f32 = -9.5;

const ANIMATION_TURN_SPEED: f32 = 1.4;
const MAX_ANIMATION_TURN: f32 = 0.4;
/// animation_rotation should be offset using this value
const ANIMATION_TURN_OFFSET: f32 = TAU - MAX_ANIMATION_TURN / 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum Activity {
    Idle,
    Walk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum TurnDirection {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
enum AnimationTurnDirection {
    Left,
    Right,
}

pub struct PenguinCreature {
    position: Vec3,
    rotation: f32,
    /// value between 0 and `MAX_ANIMATION_TURN`, should be offset with `ANIMATION_TURN_OFFSET`
    animation_rotation: f32,
    animation_turn_direction: AnimationTurnDirection,
    direction: Vec3,
    velocity: f32,
    activity_timer: ActivityTimer,
    activity: Activity,
    turn_activity_timer: ActivityTimer,
    turn_direction: TurnDirection,
    mesh: Mesh,
}
impl PenguinCreature {
    /// creates a new penguin creature at position with a random rotation
    pub fn new(position: Vec3, mesh_manager: &MeshManager) -> Self {
        let mesh = mesh_manager.create_at(MeshId::Penguin, position);
        let random_rotation = gen_range(0.0, TAU);
        let mut penguin = Self {
            position,
            velocity: 0.0,
            mesh,
            activity_timer: ActivityTimer::new(0.0, IDLE_ACTIVITY_TIME),
            direction: FORWAD_DIRECTION,
            rotation: random_rotation,
            activity: Activity::Idle,
            turn_activity_timer: ActivityTimer::new(0.0, TURN_ACTIVITY_TIME),
            turn_direction: TurnDirection::Middle,
            animation_rotation: 0.0,
            animation_turn_direction: AnimationTurnDirection::Left,
        };

        let rotate_by = (random_rotation + ANIMATION_TURN_OFFSET).rem_euclid(TAU);
        rotate_around_z_with_direction(
            &mut penguin.mesh,
            &mut penguin.direction,
            penguin.position,
            rotate_by,
        );

        penguin
    }

    /// returns true if on the ground
    fn handle_gravity(&mut self, delta: f32, world: &mut World) -> bool {
        self.velocity += delta * GRAVITY;
        self.velocity = self.velocity.min(MAX_FALL_SPEED);
        self.position.z += self.velocity * delta;
        let (new_z, is_on_ground) = collides_with_ground(self, world);

        if new_z > self.position.z || is_on_ground {
            self.velocity = 0.0;
        }
        self.position.z = new_z;

        is_on_ground
    }

    fn handle_move(&mut self, delta: f32, world: &mut World, on_ground: bool) {
        if self.activity != Activity::Walk {
            return;
        }

        let move_distance = delta * SPEED;
        let displacement = self.direction * move_distance;

        self.position += displacement;

        let collision = collides(self, world);
        if collision.is_some() {
            self.position -= displacement;
            if on_ground {
                self.velocity = JUMP;
            }
        }
    }

    /// returns turn angle
    fn handle_turn(&mut self, delta: f32) -> f32 {
        let turn_amount = match self.turn_direction {
            TurnDirection::Left => TAU - delta * TURN_SPEED,
            TurnDirection::Right => delta * TURN_SPEED,
            TurnDirection::Middle => return 0.0,
        };

        self.rotation = (self.rotation + turn_amount).rem_euclid(TAU);

        turn_amount
    }

    fn swim_if_in_water(&mut self, delta: f32, world: &mut World) {
        let voxel = world.get(vector_to_location(self.position));
        if !Voxel::WATER.contains(&voxel) {
            return;
        }

        self.velocity += delta * SWIM_SPEED;
        self.velocity = self.velocity.max(MAX_SWIM);
    }

    /// animates turning, returns turn angle (0.0-`MAX_ANIMATION_TURN`) should be offset by `ANIMATION_TURN_OFFSET`
    fn animate(&mut self, delta: f32) -> f32 {
        if self.activity != Activity::Walk {
            return 0.0;
        }

        let turn = match self.animation_turn_direction {
            AnimationTurnDirection::Left => {
                let mut turn = ANIMATION_TURN_SPEED * delta;
                if self.animation_rotation + turn >= MAX_ANIMATION_TURN {
                    turn = MAX_ANIMATION_TURN - self.animation_rotation;
                    self.animation_turn_direction = AnimationTurnDirection::Right;
                }

                turn
            }
            AnimationTurnDirection::Right => {
                let mut turn = TAU - ANIMATION_TURN_SPEED * delta;
                if self.animation_rotation + turn <= TAU {
                    turn = TAU - self.animation_rotation;
                    self.animation_turn_direction = AnimationTurnDirection::Left;
                }

                turn
            }
        };

        self.animation_rotation = (self.animation_rotation + turn).rem_euclid(TAU);

        turn
    }

    fn collide_with_player(&mut self, delta: f32, world: &mut World, player_info: &PlayerInfo) {
        if !collides_with_player(self, player_info) {
            return;
        }

        let displacement =
            push_away_from(self, player_info.camera_controller.get_position(), delta);
        self.position += displacement;
        if collides(self, world).is_some() {
            self.position -= displacement;
        }
    }
}
impl Creature for PenguinCreature {
    fn update(&mut self, delta: f32, world: &mut World, player_info: &PlayerInfo) {
        let start_position = self.position;
        let mut turn_amount = 0.0;
        if self.activity_timer.tick(delta) {
            (self.activity, self.activity_timer) = match self.activity {
                Activity::Idle => (Activity::Walk, ActivityTimer::new(0.0, WALK_ACTIVITY_TIME)),
                Activity::Walk => (Activity::Idle, ActivityTimer::new(0.0, IDLE_ACTIVITY_TIME)),
            };
        }
        if self.turn_activity_timer.tick(delta) {
            self.turn_direction = match self.turn_direction {
                TurnDirection::Left | TurnDirection::Right => TurnDirection::Middle,
                TurnDirection::Middle => {
                    if rand().is_multiple_of(2) {
                        TurnDirection::Left
                    } else {
                        TurnDirection::Right
                    }
                }
            };
        }

        let on_ground = self.handle_gravity(delta, world);
        self.handle_move(delta, world, on_ground);
        turn_amount += self.handle_turn(delta);
        turn_amount += self.animate(delta);
        self.swim_if_in_water(delta, world);
        self.collide_with_player(delta, world, player_info);
        self.position += perform_static_collisions(self, delta, world, start_position);

        let delta_position = self.position - start_position;
        move_mesh(&mut self.mesh, delta_position);
        rotate_around_z_with_direction(
            &mut self.mesh,
            &mut self.direction,
            self.position,
            turn_amount.rem_euclid(TAU),
        );
    }

    fn get_mesh_with_index(&self) -> (&Mesh, usize) {
        (&self.mesh, MeshId::Penguin.index())
    }

    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_size(&self) -> Vec3 {
        SIZE
    }

    fn create_dto(&self) -> Option<CreatureDTO> {
        let dto = PenguinDto {
            position: vec3_to_arr(self.position),
            rotation: self.rotation,
            velocity: self.velocity,
            activity_timer: self.activity_timer,
            activity: self.activity,
            turn_activity_timer: self.turn_activity_timer,
            turn_direction: self.turn_direction,
            animation_rotation: self.animation_rotation,
            animation_turn_direction: self.animation_turn_direction,
        };

        CreatureManager::encode_creature_dto(&dto, CreatureId::Penguin)
    }

    fn from_dto(creature_dto: CreatureDTO, mesh_manager: &MeshManager) -> Option<Box<dyn Creature>>
    where
        Self: Sized,
    {
        let dto: PenguinDto =
            CreatureManager::decode_creature_dto(creature_dto, CreatureId::Penguin)?;

        let position = arr_to_vec3(dto.position);
        let mut mesh = mesh_manager.create_at(MeshId::Penguin, position);
        let mut direction = FORWAD_DIRECTION;
        let total_rotation =
            (dto.rotation + dto.animation_rotation + ANIMATION_TURN_OFFSET).rem_euclid(TAU);
        rotate_around_z_with_direction(&mut mesh, &mut direction, position, total_rotation);

        let penguin = Self {
            position,
            rotation: dto.rotation,
            direction,
            velocity: dto.velocity,
            activity_timer: dto.activity_timer,
            activity: dto.activity,
            turn_activity_timer: dto.turn_activity_timer,
            turn_direction: dto.turn_direction,
            mesh,
            animation_rotation: dto.animation_rotation,
            animation_turn_direction: dto.animation_turn_direction,
        };

        Some(Box::new(penguin))
    }

    fn get_allowed_spawn_voxels() -> &'static [Voxel]
    where
        Self: Sized,
    {
        &[Voxel::Ice, Voxel::Snow]
    }
}

#[derive(Debug, Encode, Decode)]
struct PenguinDto {
    position: [f32; 3],
    rotation: f32,
    velocity: f32,
    activity_timer: ActivityTimer,
    activity: Activity,
    turn_activity_timer: ActivityTimer,
    turn_direction: TurnDirection,
    animation_rotation: f32,
    animation_turn_direction: AnimationTurnDirection,
}
