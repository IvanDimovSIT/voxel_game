use macroquad::camera::Camera3D;

use crate::{
    graphics::renderer::Renderer,
    model::{location::Location, world::World},
    service::physics::{
        falling_voxel_simulator::{FallingVoxelSimulator, SimulatedVoxelDTO},
        water_simulator::WaterSimulator,
    },
};

pub struct VoxelSimulator {
    water_simulator: WaterSimulator,
    falling_voxel_simulator: FallingVoxelSimulator,
}
impl VoxelSimulator {
    pub fn new(
        water_simulator: WaterSimulator,
        falling_voxel_simulator: FallingVoxelSimulator,
    ) -> Self {
        Self {
            water_simulator,
            falling_voxel_simulator,
        }
    }

    pub fn update(&mut self, world: &mut World, renderer: &mut Renderer, delta: f32) {
        self.falling_voxel_simulator.simulate_falling(
            world,
            renderer,
            &mut self.water_simulator,
            delta,
        );
        self.water_simulator.update(world, renderer, delta);
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

    pub fn draw(&self, camera: &Camera3D) {
        self.falling_voxel_simulator.draw(camera);
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
}
