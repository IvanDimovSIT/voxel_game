use bincode::{Decode, Encode};

use crate::{
    graphics::{
        rain_system::{RainSystem, RainSystemDTO},
        sky::{Sky, SkyDTO},
    },
    interface::tutorial_messages::{TutorialMessages, TutorialMessagesDTO},
    service::{
        creatures::creature_manager::{CreatureManager, CreatureManagerDTO},
        persistence::generic_persistence::{
            create_directory, read_binary_object, write_binary_object,
        },
        physics::{
            falling_voxel_simulator::SimulatedVoxelDTO, voxel_simulator::VoxelSimulator,
            water_simulator::WaterSimulator,
        },
        world_time::WorldTime,
    },
};

const IS_COMPRESSED: bool = false;

#[derive(Debug, Clone, Encode, Decode)]
pub struct WorldMetadata {
    pub delta: f32,
    pub simulated_voxels: Vec<SimulatedVoxelDTO>,
    pub water_simulator: WaterSimulator,
    pub creature_manager: CreatureManagerDTO,
    pub sky_dto: SkyDTO,
    pub tutorial_messages_dto: TutorialMessagesDTO,
    pub rain_system: RainSystemDTO,
}
impl WorldMetadata {
    pub fn new(
        world_time: &WorldTime,
        voxel_simulator: &VoxelSimulator,
        creature_manager: &CreatureManager,
        sky: &Sky,
        tutorial_messages: &TutorialMessages,
        rain_system: &RainSystem,
    ) -> Self {
        let (simulated_voxels, water_simulator) = voxel_simulator.create_dtos();
        Self {
            delta: world_time.get_delta(),
            simulated_voxels,
            water_simulator,
            creature_manager: creature_manager.create_dto(),
            sky_dto: sky.create_dto(),
            tutorial_messages_dto: tutorial_messages.create_dto(),
            rain_system: rain_system.create_dto(),
        }
    }
}

fn get_metadata_filepath(world_name: &str) -> String {
    format!("{world_name}/world.dat")
}

/// stores the world time
pub fn store_world_metadata(world_name: &str, world_metadata: WorldMetadata) {
    let filepath = get_metadata_filepath(world_name);
    let _create_result = create_directory(world_name);
    let _result = write_binary_object(&filepath, &world_metadata, IS_COMPRESSED);
}

/// loads the world metadata from disk
pub fn load_world_metadata(world_name: &str) -> Option<WorldMetadata> {
    let filepath = get_metadata_filepath(world_name);
    read_binary_object(&filepath, IS_COMPRESSED)
}
