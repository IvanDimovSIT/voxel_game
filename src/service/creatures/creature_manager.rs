use macroquad::{
    camera::Camera3D,
    math::{Vec3, vec3},
    prelude::info,
    rand::gen_range,
};

use crate::{
    graphics::mesh_manager::MeshManager,
    model::{
        area::AREA_SIZE, location::Location, player_info::PlayerInfo, user_settings::UserSettings,
        world::World,
    },
    service::{activity_timer::ActivityTimer, creatures::bunny_creature::BunnyCreature},
    utils::vector_to_location,
};

const CHECK_UPDATES_TIME: f32 = 10.0;
const REMOVE_RANGE: f32 = 300.0;
const MAX_SPAWN_DISTANCE: f32 = 100.0;
const MAX_CREATURES: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureId {
    Bunny,
}

pub trait Creature {
    fn update(&mut self, delta: f32, world: &mut World);
    fn draw(&self);
    fn get_position(&self) -> Vec3;
    fn get_size(&self) -> Vec3;
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

    pub fn update(
        &mut self,
        delta: f32,
        mesh_manager: &MeshManager,
        player_info: &PlayerInfo,
        world: &mut World,
    ) {
        for creature in &mut self.creatures {
            creature.update(delta, world);
        }

        if self.activity_timer.tick(delta) {
            let camera = player_info.camera_controller.create_camera();
            let camera_look = (camera.target - camera.position).normalize_or_zero();
            self.remove_distant_creatures(&camera, camera_look);
            if self.creatures.len() >= MAX_CREATURES {
                return;
            }
            self.add_creature(mesh_manager, world, &camera, camera_look);
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
        for creature in &self.creatures {
            let creature_pos = creature.get_position();
            let vec_to_creature = creature_pos - camera.position;
            let distance_to_creature = vec_to_creature.length();
            if distance_to_creature > draw_cull_range {
                continue;
            }
            if vec_to_creature.normalize_or_zero().dot(camera_look) < 0.0
                && distance_to_creature > 3.0
            {
                continue;
            }

            creature.draw();
            drew += 1;
        }

        drew
    }

    pub fn collides(creature: &impl Creature, world: &mut World) -> bool {
        let pos = creature.get_position();
        let size = creature.get_size();
        let creature_location: Location = pos.into();
        let area = world.take_area(creature_location.into());
        let cached_area = Some(&area);

        let has_collision = world
            .get_with_cache(creature_location, cached_area)
            .is_solid()
            || {
                let positions_to_check = [
                    pos + vec3(size.x * 0.5, 0.0, 0.0),
                    pos - vec3(size.x * 0.5, 0.0, 0.0),
                    pos + vec3(0.0, size.y * 0.5, 0.0),
                    pos - vec3(0.0, size.y * 0.5, 0.0),
                ];

                positions_to_check.into_iter().any(|loc| {
                    world
                        .get_with_cache(Into::<Location>::into(loc), cached_area)
                        .is_solid()
                })
            };

        world.return_area(area);
        has_collision
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

        let area = world.take_area(bottom_location.into());
        let cached_area = Some(&area);
        let bottom_voxel = world.get_with_cache(bottom_location, cached_area);

        if !bottom_voxel.is_solid() {
            let top_voxel = world.get_with_cache(top_location, cached_area);
            let result = if top_voxel.is_solid() {
                top_location.z as f32 + 0.5 + half_z
            } else {
                position.z
            };

            world.return_area(area);
            return (result, false);
        }

        world.return_area(area);
        (bottom_location.z as f32 - 0.5 - half_z, true)
    }

    fn remove_distant_creatures(&mut self, camera: &Camera3D, camera_look: Vec3) {
        self.creatures.retain(|creature| {
            let creature_pos = creature.get_position();
            let vec_to_creature = creature_pos - camera.position;
            let distance_to_creature = vec_to_creature.length();
            if distance_to_creature < REMOVE_RANGE {
                return true;
            }

            vec_to_creature.normalize_or_zero().dot(camera_look) < 0.0 && distance_to_creature > 5.0
        });
    }

    fn add_creature(
        &mut self,
        mesh_manager: &MeshManager,
        world: &mut World,
        camera: &Camera3D,
        camera_look: Vec3,
    ) {
        let random_x = gen_range(-MAX_SPAWN_DISTANCE, MAX_SPAWN_DISTANCE);
        let random_y = gen_range(-MAX_SPAWN_DISTANCE, MAX_SPAWN_DISTANCE);
        let location = vector_to_location(vec3(
            camera.position.x + random_x,
            camera.position.y + random_y,
            0.0,
        ));
        let camera_to_location = Into::<Vec3>::into(location) - camera.position;
        if camera_to_location.normalize().dot(camera_look) > 0.0 {
            info!("No creatures added");
            return;
        }

        let (area_loc, local) = World::convert_global_to_area_and_local_location(location.into());
        let height = world
            .get_area_without_loading(area_loc)
            .sample_height(local.x, local.y);

        let creature_location = vec3(location.x as f32, location.y as f32, height as f32 - 1.0);
        let mesh = mesh_manager.get_at(CreatureId::Bunny, creature_location);
        let creature = Box::new(BunnyCreature::new(creature_location, mesh));
        self.creatures.push(creature);
        info!("Added creature at {}", camera_to_location);
    }
}
