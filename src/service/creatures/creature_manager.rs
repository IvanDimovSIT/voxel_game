use bincode::{Decode, Encode, decode_from_slice, encode_to_vec};
use macroquad::{
    camera::Camera3D,
    math::{Vec3, vec3},
    models::{Mesh, draw_mesh},
    prelude::{error, info},
    rand::{gen_range, rand},
};

use crate::{
    graphics::mesh_manager::{MeshId, MeshManager},
    model::{
        area::AREA_SIZE, location::Location, player_info::PlayerInfo, user_settings::UserSettings,
        voxel::Voxel, world::World,
    },
    service::{
        activity_timer::ActivityTimer,
        creatures::{bunny_creature::BunnyCreature, butterfly_creature::ButterflyCreature},
        persistence::config::SERIALIZATION_CONFIG,
    },
    utils::vector_to_location,
};

const CHECK_UPDATES_TIME: f32 = 3.0;
const MAX_CREATURES: usize = 10;
const SPAWN_SIZE_EXTRA_RANGE: f32 = AREA_SIZE as f32 * 0.75;
const MIN_CULL_DISTANCE: f32 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum CreatureId {
    Bunny,
    Butterfly,
}
impl CreatureId {
    pub const VARIANTS: usize = 2;
}
impl From<u32> for CreatureId {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Bunny,
            1 => Self::Butterfly,
            _ => panic!("Invalid index for CreatureId"),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreatureDTO {
    pub id: CreatureId,
    pub bytes: Vec<u8>,
}

pub trait Creature {
    fn update(&mut self, delta: f32, world: &mut World);
    fn get_mesh_with_index(&self) -> (&Mesh, usize);
    fn get_position(&self) -> Vec3;
    fn get_size(&self) -> Vec3;
    fn create_dto(&self) -> Option<CreatureDTO>;
    fn from_dto(creature_dto: CreatureDTO, mesh_manager: &MeshManager) -> Option<Box<dyn Creature>>
    where
        Self: Sized;
}

pub struct CreatureManager {
    creatures: Vec<Box<dyn Creature>>,
    activity_timer: ActivityTimer,
}
impl CreatureManager {
    pub fn new() -> Self {
        Self {
            creatures: vec![],
            activity_timer: ActivityTimer::new(0.0, CHECK_UPDATES_TIME),
        }
    }

    pub fn from_dto(dto: CreatureManagerDTO, mesh_manager: &MeshManager) -> Self {
        let creatures = dto
            .creatures
            .into_iter()
            .flat_map(|creature_dto| match creature_dto.id {
                CreatureId::Bunny => BunnyCreature::from_dto(creature_dto, mesh_manager),
                CreatureId::Butterfly => ButterflyCreature::from_dto(creature_dto, mesh_manager),
            })
            .collect();

        Self {
            creatures,
            activity_timer: ActivityTimer::new(dto.activity_delta, CHECK_UPDATES_TIME),
        }
    }

    pub fn update(
        &mut self,
        delta: f32,
        mesh_manager: &MeshManager,
        player_info: &PlayerInfo,
        world: &mut World,
        user_settings: &UserSettings,
    ) {
        let creature_spawn_distance =
            user_settings.get_render_distance() as f32 * AREA_SIZE as f32 + SPAWN_SIZE_EXTRA_RANGE;
        for creature in &mut self.creatures {
            creature.update(delta, world);
        }
        self.remove_distant_creatures(
            player_info.camera_controller.get_position(),
            creature_spawn_distance,
        );

        if self.activity_timer.tick(delta) && self.creatures.len() < MAX_CREATURES {
            let camera = player_info.camera_controller.create_camera();
            let camera_look = (camera.target - camera.position).normalize_or_zero();
            self.add_creature(
                mesh_manager,
                world,
                &camera,
                camera_look,
                creature_spawn_distance,
            );
        }
    }

    pub fn check_can_place_voxel(&self, location: Location) -> bool {
        let voxel_position: Vec3 = location.into();
        self.creatures.iter().all(|creature| {
            let creature_position = creature.get_position();
            let size = creature.get_size();
            let offset = creature_position - voxel_position;

            !(-size.x / 2.0..size.x / 2.0).contains(&offset.x)
                || !(-size.y / 2.0..size.y / 2.0).contains(&offset.y)
                || !(-size.z / 2.0..size.z / 2.0).contains(&offset.z)
        })
    }

    /// draws all visible creatures and returns the number drawn
    pub fn draw(&self, camera: &Camera3D, user_settings: &UserSettings) -> u32 {
        if self.creatures.is_empty() {
            return 0;
        }
        let camera_look = camera.target - camera.position;
        let draw_cull_range = (user_settings.get_render_distance() * AREA_SIZE) as f32;

        let mut drew = 0;
        let mut mesh_array = vec![vec![]; MeshId::VARIANTS];
        for creature in &self.creatures {
            let creature_pos = creature.get_position();
            let vec_to_creature = creature_pos - camera.position;
            let distance_to_creature = vec_to_creature.length();
            if distance_to_creature > draw_cull_range {
                continue;
            }
            if distance_to_creature > MIN_CULL_DISTANCE
                && vec_to_creature.normalize_or_zero().dot(camera_look) < 0.0
            {
                continue;
            }

            let (mesh, index) = creature.get_mesh_with_index();
            mesh_array[index].push(mesh);
            drew += 1;
        }

        Self::draw_mesh_array(mesh_array);

        drew
    }

    fn draw_mesh_array(mesh_array: Vec<Vec<&Mesh>>) {
        let ordered_meshes = mesh_array.into_iter().flatten();
        for mesh in ordered_meshes {
            draw_mesh(mesh);
        }
    }

    pub fn collides(creature: &impl Creature, world: &mut World) -> bool {
        let pos = creature.get_position();
        let size = creature.get_size();
        let creature_location: Location = pos.into();

        world.with_cached_area(creature_location, |world, cached_area| {
            let positions_to_check = [
                pos,
                pos + vec3(size.x * 0.5, 0.0, 0.0),
                pos - vec3(size.x * 0.5, 0.0, 0.0),
                pos + vec3(0.0, size.y * 0.5, 0.0),
                pos - vec3(0.0, size.y * 0.5, 0.0),
            ];

            positions_to_check.into_iter().any(|loc| {
                world
                    .get_with_cache(Into::<Location>::into(loc), Some(cached_area))
                    .is_solid()
            })
        })
    }

    /// returns new creature z and if it's on the ground
    pub fn collides_with_ground(creature: &impl Creature, world: &mut World) -> (f32, bool) {
        let position = creature.get_position();
        let size = creature.get_size();
        let half_z = size.z * 0.5;
        let below = vec3(position.x, position.y, position.z + half_z);
        let above = vec3(position.x, position.y, position.z - half_z);

        let bottom_location = vector_to_location(below);
        let top_location = vector_to_location(above);

        let bottom_voxel = world.get(bottom_location);

        if !bottom_voxel.is_solid() {
            let top_voxel = world.get(top_location);
            let result = if top_voxel.is_solid() {
                top_location.z as f32 + Voxel::HALF_SIZE + half_z
            } else {
                position.z
            };

            return (result, false);
        }

        (bottom_location.z as f32 - Voxel::HALF_SIZE - half_z, true)
    }

    pub fn create_dto(&self) -> CreatureManagerDTO {
        let creatures = self
            .creatures
            .iter()
            .flat_map(|creature| creature.create_dto())
            .collect();

        CreatureManagerDTO {
            creatures,
            activity_delta: self.activity_timer.get_delta(),
        }
    }

    pub fn creature_count(&self) -> usize {
        self.creatures.len()
    }

    pub fn encode_creature_dto(custom_dto: &impl Encode, id: CreatureId) -> Option<CreatureDTO> {
        let serialisation_result = encode_to_vec(custom_dto, SERIALIZATION_CONFIG);
        match serialisation_result {
            Ok(bytes) => Some(CreatureDTO { id, bytes }),
            Err(err) => {
                error!("Error serialising '{:?}': {}", id, err);
                None
            }
        }
    }

    pub fn decode_creature_dto<T>(creature_dto: CreatureDTO, id: CreatureId) -> Option<T>
    where
        T: Decode<()>,
    {
        if creature_dto.id != id {
            error!(
                "Creature id {:?} doesn't match expected {:?}",
                creature_dto.id, id
            );
            return None;
        }
        let decode_result: Result<(T, _), _> =
            decode_from_slice(&creature_dto.bytes, SERIALIZATION_CONFIG);

        match decode_result {
            Ok((dto, _size)) => Some(dto),
            Err(err) => {
                error!("Error decoding creature {:?}: '{}'", id, err);
                None
            }
        }
    }

    fn remove_distant_creatures(&mut self, camera_pos: Vec3, creature_spawn_distance: f32) {
        let creature_count = self.creatures.len();
        self.creatures.retain(|creature| {
            let creature_pos = creature.get_position();
            let distance_to_creature = camera_pos.distance(creature_pos);
            distance_to_creature <= creature_spawn_distance
        });
        let removed_creatures = creature_count as i32 - self.creatures.len() as i32;
        if removed_creatures != 0 {
            info!("Removed {} creature(s)", removed_creatures);
        }
    }

    fn add_creature(
        &mut self,
        mesh_manager: &MeshManager,
        world: &mut World,
        camera: &Camera3D,
        camera_look: Vec3,
        render_distance: f32,
    ) {
        let random_x = gen_range(-render_distance, render_distance);
        let random_y = gen_range(-render_distance, render_distance);
        let location = vector_to_location(vec3(
            camera.position.x + random_x,
            camera.position.y + random_y,
            0.0,
        ));
        let height = world.get_height(location);
        let location = Location {
            z: height as i32,
            ..location
        };
        let camera_to_location = Into::<Vec3>::into(location) - camera.position;
        if camera_to_location.normalize().dot(camera_look) > 0.0 {
            info!("No creatures added");
            return;
        }

        let creature_position = vec3(
            location.x as f32,
            location.y as f32,
            (height as f32 - 1.0).max(0.0),
        );
        let id = (rand() % CreatureId::VARIANTS as u32).into();
        let creature = Self::create_creature(id, creature_position, mesh_manager);
        self.creatures.push(creature);
        info!("Added creature at {}", camera_to_location);
    }

    fn create_creature(
        id: CreatureId,
        position: Vec3,
        mesh_manager: &MeshManager,
    ) -> Box<dyn Creature> {
        match id {
            CreatureId::Bunny => Box::new(BunnyCreature::new(position, mesh_manager)),
            CreatureId::Butterfly => Box::new(ButterflyCreature::new(position, mesh_manager)),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct CreatureManagerDTO {
    creatures: Vec<CreatureDTO>,
    activity_delta: f32,
}
