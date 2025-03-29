use macroquad::{conf::Conf, time::get_frame_time};
use voxel_engine::VoxelEngine;

mod graphics;
mod model;
mod service;
mod utils;
mod voxel_engine;

fn config() -> Conf {
    Conf {
        draw_call_vertex_capacity: 400,
        draw_call_index_capacity: 200,
        ..Default::default()        
    }
}

#[macroquad::main("Voxel World", config)]
async fn main() {
    let mut voxel_engine = VoxelEngine::new("test_world").await;

    loop {
        let delta = get_frame_time();
        let raycast_result = voxel_engine.process_input(delta);
        voxel_engine.update_loaded_areas();
        voxel_engine.draw_scene(raycast_result).await;
    }
}
