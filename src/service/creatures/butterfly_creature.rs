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
        mesh_transformer::{move_mesh, rotate_around_z, rotate_around_z_with_direction},
    },
    model::{player_info::PlayerInfo, voxel::Voxel, world::World},
    service::{
        activity_timer::ActivityTimer,
        creatures::{
            creature::{Creature, collides, collides_with_ground, push_away_from},
            creature_manager::{CreatureDTO, CreatureId, CreatureManager},
        },
    },
    utils::{arr_to_vec3, vec3_to_arr, vector_to_location},
};

const SIZE: Vec3 = vec3(0.5, 0.5, 0.3);
const FORWAD_DIRECTION: Vec3 = vec3(0.0, 1.0, 0.0);
const DISTANCE_FROM_GROUND: f32 = 3.0;
const FLY_UP: f32 = -12.0;
const FLY_DOWN: f32 = 8.0;
const MAX_MIN_VELOCITY: f32 = 4.0;
const HORIZONTAL_SPEED: f32 = 4.0;

const WING_FLAP_DELAY: f32 = 0.15;

const MIN_TURN_TIME: f32 = 1.0;
const MAX_TURN_TIME: f32 = 4.0;
const TURN_SPEED: f32 = 0.5;

#[derive(Debug, Clone, Copy, Encode, Decode)]
enum TurnDirection {
    Left,
    Right,
    Middle,
}

fn random_turn_time() -> f32 {
    gen_range(MIN_TURN_TIME, MAX_TURN_TIME)
}

pub struct ButterflyCreature {
    position: Vec3,
    direction: Vec3,
    angle: f32,
    velocity: f32,
    wing_flap_activity: ActivityTimer,
    mesh_arr: [(Mesh, MeshId); 2],
    current_mesh: usize,
    turn_activity: ActivityTimer,
    turn_direction: TurnDirection,
}
impl ButterflyCreature {
    pub fn new(position: Vec3, mesh_manager: &MeshManager) -> Self {
        let mesh1 = mesh_manager.create_at(MeshId::ButterflyDown, position);
        let mesh2 = mesh_manager.create_at(MeshId::ButterflyUp, position);
        let mesh_arr = [(mesh1, MeshId::ButterflyDown), (mesh2, MeshId::ButterflyUp)];

        Self {
            position,
            direction: FORWAD_DIRECTION,
            velocity: -0.1,
            mesh_arr,
            current_mesh: 0,
            wing_flap_activity: ActivityTimer::new(0.0, WING_FLAP_DELAY),
            turn_direction: TurnDirection::Middle,
            turn_activity: ActivityTimer::new(0.0, random_turn_time()),
            angle: 0.0,
        }
    }

    fn fly(&mut self, delta: f32, world: &mut World) {
        if self.should_fly(world) {
            self.velocity += delta * FLY_UP;
        } else {
            self.velocity += delta * FLY_DOWN;
        }
        self.velocity = self.velocity.clamp(-MAX_MIN_VELOCITY, MAX_MIN_VELOCITY);

        self.position.z += self.velocity * delta;

        let displacement = self.direction * HORIZONTAL_SPEED * delta;
        self.position += displacement;
        let collision = collides(self, world);
        if let Some(point) = collision {
            self.position -= displacement;
            self.position += push_away_from(self, point, delta);
        }
    }

    fn should_fly(&self, world: &mut World) -> bool {
        let location_front = vector_to_location(self.direction + self.position);
        let location = vector_to_location(Vec3 {
            z: self.position.z + DISTANCE_FROM_GROUND,
            ..self.position
        });
        let voxel_height = world.get_height(location);

        if self.position.z + DISTANCE_FROM_GROUND > voxel_height as f32 {
            return true;
        }

        if world.get(location_front).is_solid() {
            return true;
        }

        let voxel = world.get(location);
        Voxel::WATER.contains(&voxel)
    }

    fn animate(&mut self, delta: f32) {
        if self.wing_flap_activity.tick(delta) {
            self.current_mesh = (self.current_mesh + 1) % self.mesh_arr.len();
        }
    }

    fn turn(&mut self, delta: f32) {
        if self
            .turn_activity
            .tick_change_cooldown(delta, random_turn_time)
        {
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

        let turn_angle = match self.turn_direction {
            TurnDirection::Left => TURN_SPEED * delta,
            TurnDirection::Right => TAU - TURN_SPEED * delta,
            TurnDirection::Middle => return,
        };
        self.angle += turn_angle;
        if self.angle > TAU {
            self.angle -= TAU;
        }

        rotate_around_z_with_direction(
            &mut self.mesh_arr[0].0,
            &mut self.direction,
            self.position,
            turn_angle,
        );
        rotate_around_z(&mut self.mesh_arr[1].0, self.position, turn_angle);
    }
}
impl Creature for ButterflyCreature {
    fn update(&mut self, delta: f32, world: &mut World, _player_info: &PlayerInfo) {
        let original_position = self.position;
        self.turn(delta);
        self.fly(delta, world);
        let (new_z, _is_on_ground) = collides_with_ground(self, world);

        self.animate(delta);

        self.position.z = new_z;
        let delta_position = self.position - original_position;
        for (mesh, _id) in &mut self.mesh_arr {
            move_mesh(mesh, delta_position);
        }
    }

    fn get_mesh_with_index(&self) -> (&Mesh, usize) {
        let (mesh, id) = &self.mesh_arr[self.current_mesh];
        (mesh, id.index())
    }

    fn get_position(&self) -> Vec3 {
        self.position
    }

    fn get_size(&self) -> Vec3 {
        SIZE
    }

    fn create_dto(&self) -> Option<CreatureDTO> {
        let dto = ButterflyDto {
            position: vec3_to_arr(self.position),
            velocity: self.velocity,
            wing_flap_delta: self.wing_flap_activity.get_delta(),
            current_mesh: self.current_mesh,
            turn_activity: self.turn_activity,
            turn_direction: self.turn_direction,
            angle: self.angle,
        };

        CreatureManager::encode_creature_dto(&dto, CreatureId::Butterfly)
    }

    fn from_dto(
        creature_dto: CreatureDTO,
        mesh_manager: &MeshManager,
    ) -> Option<Box<dyn Creature>> {
        let butterfly_dto: ButterflyDto =
            CreatureManager::decode_creature_dto(creature_dto, CreatureId::Butterfly)?;
        let position = arr_to_vec3(butterfly_dto.position);
        let mut direction = FORWAD_DIRECTION;
        let angle = butterfly_dto.angle;

        let mut mesh1 = mesh_manager.create_at(MeshId::ButterflyDown, position);
        let mut mesh2 = mesh_manager.create_at(MeshId::ButterflyUp, position);
        rotate_around_z_with_direction(&mut mesh1, &mut direction, position, angle);
        rotate_around_z(&mut mesh2, position, angle);
        let mesh_arr = [(mesh1, MeshId::ButterflyDown), (mesh2, MeshId::ButterflyUp)];

        let butterfly = Self {
            position,
            direction,
            angle,
            velocity: butterfly_dto.velocity,
            wing_flap_activity: ActivityTimer::new(butterfly_dto.wing_flap_delta, WING_FLAP_DELAY),
            mesh_arr,
            current_mesh: butterfly_dto.current_mesh,
            turn_activity: butterfly_dto.turn_activity,
            turn_direction: butterfly_dto.turn_direction,
        };

        Some(Box::new(butterfly))
    }

    fn get_allowed_spawn_voxels() -> &'static [Voxel]
    where
        Self: Sized,
    {
        &[Voxel::Grass, Voxel::Clay, Voxel::Leaves]
    }
}

#[derive(Debug, Encode, Decode)]
struct ButterflyDto {
    position: [f32; 3],
    angle: f32,
    velocity: f32,
    wing_flap_delta: f32,
    current_mesh: usize,
    turn_activity: ActivityTimer,
    turn_direction: TurnDirection,
}
