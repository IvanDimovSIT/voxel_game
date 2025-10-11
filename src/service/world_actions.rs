use std::rc::Rc;

use macroquad::math::vec3;

use crate::{
    graphics::{renderer::Renderer, sky::Sky, voxel_particle_system::VoxelParticleSystem},
    model::{
        area::AREA_HEIGHT, location::Location, player_info::PlayerInfo, voxel::Voxel, world::World,
    },
    service::{
        asset_manager::AssetManager,
        creatures::creature_manager::CreatureManager,
        persistence::{
            player_persistence::load_player_info, world_metadata_persistence::load_world_metadata,
        },
        physics::{
            falling_voxel_simulator::FallingVoxelSimulator,
            player_physics::will_new_voxel_cause_collision, voxel_simulator::VoxelSimulator,
            water_simulator::WaterSimulator,
        },
        world_time::WorldTime,
    },
    utils::vector_to_location,
};

pub fn place_voxel(
    location: Location,
    voxel: Voxel,
    player_info: &PlayerInfo,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
    creature_manager: &CreatureManager,
) -> bool {
    debug_assert!(voxel != Voxel::None);
    let unable_to_place_voxel = world.get(location).is_solid()
        || will_new_voxel_cause_collision(player_info, location)
        || voxel_simulator.location_has_voxel(location)
        || !creature_manager.check_can_place_voxel(location);

    if unable_to_place_voxel {
        return false;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    true
}

pub fn replace_voxel(
    location: Location,
    voxel: Voxel,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
) -> Option<Voxel> {
    debug_assert!(voxel != Voxel::None);
    let to_be_replaced = world.get(location);
    if !to_be_replaced.is_solid() || location.z == AREA_HEIGHT as i32 - 1 || to_be_replaced == voxel
    {
        return None;
    }

    world.set(location, voxel);
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    Some(to_be_replaced)
}

pub fn destroy_voxel(
    location: Location,
    world: &mut World,
    renderer: &mut Renderer,
    voxel_simulator: &mut VoxelSimulator,
    voxel_particles: &mut VoxelParticleSystem,
) -> Option<Voxel> {
    let voxel = world.get(location);
    if voxel == Voxel::None || location.z == AREA_HEIGHT as i32 - 1 {
        return None;
    }

    world.set(location, Voxel::None);
    voxel_particles.add_particles_for_destroyed(voxel, location, renderer.get_mesh_generator());
    renderer.update_location(world, location);
    voxel_simulator.update_location(location, world, renderer);

    Some(voxel)
}

pub fn update_player_in_water(player_info: &mut PlayerInfo, world: &mut World) {
    let player_location = player_info.camera_controller.get_camera_voxel_location();
    player_info.is_in_water = Voxel::WATER.contains(&world.get(player_location));
}

/// struct containing the loaded systems for the voxel engine
pub struct WorldSystems {
    pub world_time: WorldTime,
    pub renderer: Renderer,
    pub world: World,
    pub voxel_simulator: VoxelSimulator,
    pub creature_manager: CreatureManager,
    pub player_info: PlayerInfo,
    pub sky: Sky,
}

/// loads the saved world data or initialises it if not saved
pub fn initialise_world_systems(
    world_name: impl Into<String>,
    asset_manager: Rc<AssetManager>,
) -> WorldSystems {
    let world_name = world_name.into();
    let (mut player_info, successful_load) = load_player_info(&world_name)
        .map(|info| (info, true))
        .unwrap_or_else(|| (PlayerInfo::new(vec3(0.0, 0.0, 0.0)), false));

    player_info.camera_controller.set_focus(true);
    let (world_time, simulated_voxels, water_simulator, creature_manager, sky) =
        if let Some(world_metadata) = load_world_metadata(&world_name) {
            (
                WorldTime::new(world_metadata.delta),
                world_metadata.simulated_voxels,
                world_metadata.water_simulator,
                CreatureManager::from_dto(
                    world_metadata.creature_manager,
                    &asset_manager.mesh_manager,
                ),
                Sky::from_dto(&asset_manager.texture_manager, world_metadata.sky_dto),
            )
        } else {
            (
                WorldTime::new(std::f32::consts::PI * 0.5),
                vec![],
                WaterSimulator::new(),
                CreatureManager::new(),
                Sky::new(&asset_manager.texture_manager),
            )
        };

    let renderer = Renderer::new(asset_manager.clone());
    let falling_voxel_simulator =
        FallingVoxelSimulator::new(simulated_voxels, renderer.get_mesh_generator());
    let voxel_simulator = VoxelSimulator::new(water_simulator, falling_voxel_simulator);
    let mut world = World::new(world_name);

    if !successful_load {
        put_player_on_ground(&mut player_info, &mut world);
    }

    WorldSystems {
        world_time,
        renderer,
        world,
        voxel_simulator,
        creature_manager,
        sky,
        player_info,
    }
}

fn put_player_on_ground(player_info: &mut PlayerInfo, world: &mut World) {
    loop {
        let bottom_position = player_info.camera_controller.get_bottom_position();
        let bottom_location = vector_to_location(bottom_position);
        if bottom_location.z + 1 >= AREA_HEIGHT as i32 {
            return;
        }
        let voxel = world.get(bottom_location);

        if voxel != Voxel::None {
            player_info
                .camera_controller
                .set_position(bottom_position - vec3(0.0, 0.0, 1.0));

            if Voxel::WATER.contains(&voxel) {
                world.set(bottom_location, Voxel::Stone);
            }
            return;
        } else {
            player_info
                .camera_controller
                .set_position(bottom_position + vec3(0.0, 0.0, 1.0));
        }
    }
}
