use macroquad::{camera::Camera3D, math::Vec3};

use crate::{
    graphics::renderer::Renderer,
    model::{location::Location, user_settings::UserSettings, world::World},
    service::{
        asset_manager::AssetManager,
        physics::{
            bomb_simulator::BombSimulator,
            falling_voxel_simulator::{FallingVoxelSimulator, SimulatedVoxelDTO},
            water_simulator::WaterSimulator,
        },
    },
};

pub struct VoxelSimulator {
    water_simulator: WaterSimulator,
    falling_voxel_simulator: FallingVoxelSimulator,
    bomb_simulator: BombSimulator,
}
impl VoxelSimulator {
    pub fn new(
        water_simulator: WaterSimulator,
        falling_voxel_simulator: FallingVoxelSimulator,
    ) -> Self {
        Self {
            water_simulator,
            falling_voxel_simulator,
            bomb_simulator: BombSimulator::new(),
        }
    }

    pub fn update(
        &mut self,
        world: &mut World,
        renderer: &mut Renderer,
        asset_manager: &AssetManager,
        user_settings: &UserSettings,
        delta: f32,
    ) {
        self.falling_voxel_simulator.simulate_falling(
            world,
            renderer,
            &mut self.water_simulator,
            delta,
        );
        self.water_simulator.update(world, renderer, delta);
        let updated_locations =
            self.bomb_simulator
                .update(world, renderer, asset_manager, user_settings, delta);
        for loc in updated_locations {
            self.update_location(loc, world, renderer);
        }
    }

    pub fn update_location(
        &mut self,
        location: Location,
        world: &mut World,
        renderer: &mut Renderer,
    ) {
        self.falling_voxel_simulator.update_voxels(
            world,
            renderer,
            &mut self.water_simulator,
            location,
        );
        self.water_simulator.location_updated(location);
    }

    /// draws elements that require the standard voxel shader
    pub fn draw_for_voxel_shader(&self, camera: &Camera3D, renderer: &Renderer) {
        self.falling_voxel_simulator.draw(camera);
        self.bomb_simulator.draw_bombs(renderer);
    }

    /// returns a vector of explosion locations
    pub fn draw_for_flat_shader(&self, camera: &Camera3D) -> Vec<Vec3> {
        self.bomb_simulator.draw_explosions(camera)
    }

    pub fn location_has_voxel(&self, location: Location) -> bool {
        self.falling_voxel_simulator.location_has_voxel(location)
    }

    pub fn create_dtos(&self) -> (Vec<SimulatedVoxelDTO>, WaterSimulator) {
        (
            self.falling_voxel_simulator.create_simulated_voxel_dtos(),
            self.water_simulator.clone(),
        )
    }

    pub fn add_bomb(&mut self, location: Location) {
        self.bomb_simulator.add_active_bomb(location);
    }
}
