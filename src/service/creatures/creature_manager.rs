use macroquad::{camera::Camera3D, math::{vec3, Vec3}, prelude::info, rand::gen_range};

use crate::{graphics::mesh_manager::{MeshId, MeshManager}, model::{area::AREA_SIZE, location::{self, Location}, player_info::{self, PlayerInfo}, user_settings::UserSettings, world::World}, service::{activity_timer::ActivityTimer, creatures::test_creature::TestCreature}, utils::vector_to_location};

const CHECK_UPDATES_TIME: f32 = 0.1;
const REMOVE_RANGE: f32 = 300.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureId {
    Test
}

pub trait Creature {
    fn update(&mut self, delta: f32, world: &mut World);
    fn draw(&self);
    fn get_position(&self) -> Vec3;
    fn get_size(&self) -> Vec3;
}

pub struct CreatureManager {
    creatures: Vec<Box<dyn Creature>>,
    activity_timer: ActivityTimer
}
impl CreatureManager {
    pub fn new() -> Self {
        Self { creatures: vec![], activity_timer: ActivityTimer::new(0.0, CHECK_UPDATES_TIME) }
    }

    pub fn update(&mut self, delta: f32, mesh_manager: &MeshManager, player_info: &PlayerInfo, world: &mut World) {
        for creature in &mut self.creatures {
            creature.update(delta, world);
        }

        if self.activity_timer.tick(delta) {
            let camera = player_info.camera_controller.create_camera();
            let camera_look = (camera.target - camera.position)
                .normalize_or_zero();
            self.remove_distant_creatures(&camera, camera_look);
            if self.creatures.len() > 20 {
                return;
            }
            self.add_creature(mesh_manager, world, &camera, camera_look);
        }
    }

    pub fn check_can_place_voxel(&self, location: Location) -> bool {
        let voxel_position: Vec3 = location.into(); 
        self.creatures.iter()
            .all(|creature| {
                let creature_position = creature.get_position();
                let size = creature.get_size();
                let offset = creature_position - voxel_position;
                
                !(-size.x/2.0..size.x/2.0).contains(&offset.x) ||
                !(-size.y/2.0..size.y/2.0).contains(&offset.y) ||
                !(-size.z/2.0..size.z/2.0).contains(&offset.z)
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
            if vec_to_creature.normalize_or_zero()
                .dot(camera_look) < 0.0 && distance_to_creature > 5.0 {
                continue;
            }

            creature.draw();
            drew += 1;
        }

        drew
    }

    fn remove_distant_creatures(&mut self, camera: &Camera3D, camera_look: Vec3) {
        self.creatures.retain(|creature| {
            let creature_pos = creature.get_position();
            let vec_to_creature = creature_pos - camera.position;
            let distance_to_creature = vec_to_creature.length(); 
            if distance_to_creature < REMOVE_RANGE {
                return true;
            }
            
            vec_to_creature.normalize_or_zero()
                .dot(camera_look) < 0.0 && distance_to_creature > 5.0
        });
    }

    fn add_creature(&mut self, mesh_manager: &MeshManager, world: &mut World, camera: &Camera3D, camera_look: Vec3) {
        let random_x = gen_range(-100.0, 100.0);
        let random_y = gen_range(-100.0, 100.0);
        let location = vector_to_location(vec3(camera.position.x + random_x, camera.position.y + random_y, 0.0));
        let camera_to_location = camera.position - Into::<Vec3>::into(location);
        if camera_to_location.normalize().dot(camera_look) > 0.0 {
            return;
        }

        let (area_loc, local) = World::convert_global_to_area_and_local_location(location.into());
        let height = world.get_area_without_loading(area_loc).sample_height(local.x, local.y);

        let creature_location = vec3(location.x as f32, location.y as f32, height as f32 - 1.0);
        let mesh = mesh_manager.get_at(MeshId::TestModel, creature_location);
        let creature = Box::new(TestCreature::new(creature_location, mesh));
        self.creatures.push(creature);
        info!("Added creature at {}", camera_to_location);
    }
}