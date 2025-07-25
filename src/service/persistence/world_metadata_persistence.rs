use std::fs::create_dir_all;

use bincode::{Decode, Encode};

use crate::service::{
    persistence::generic_persistence::{read_binary_object, write_binary_object},
    physics::voxel_physics::{SimulatedVoxelDTO, VoxelSimulator},
    world_time::WorldTime,
};

#[derive(Debug, Clone, Encode, Decode)]
pub struct WorldMetadata {
    pub delta: f32,
    pub simulated_voxels: Vec<SimulatedVoxelDTO>,
}
impl WorldMetadata {
    pub fn new(world_time: &WorldTime, voxel_simulator: &VoxelSimulator) -> Self {
        Self {
            delta: world_time.get_delta(),
            simulated_voxels: voxel_simulator.create_simulated_voxel_dtos(),
        }
    }
}

fn get_metadata_filepath(world_name: &str) -> String {
    format!("{world_name}/world.dat")
}

/// stores the world time
pub fn store_world_metadata(world_metadata: WorldMetadata, world_name: &str) {
    let filepath = get_metadata_filepath(world_name);
    let _ = create_dir_all(world_name);
    let _result = write_binary_object(&filepath, &world_metadata);
}

/// loads the world metadata from disk
pub fn load_world_metadata(world_name: &str) -> Option<WorldMetadata> {
    let filepath = get_metadata_filepath(world_name);
    read_binary_object(&filepath)
}
