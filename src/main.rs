use graphics::renderer::Renderer;
use macroquad::{
    camera::{Camera3D, set_camera},
    color::{BEIGE, Color},
    logging,
    math::vec3,
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use model::{
    location::{InternalLocation, LOCATION_OFFSET, Location},
    world::World,
};
use service::{
    camera_controller::{self, CameraController},
    input::{enter_focus, exit_focus, move_back, move_forward, move_left, move_right},
    render_zone::{get_load_zone, get_render_zone},
};

mod graphics;
mod model;
mod service;
mod utils;

const RENDER_SIZE: u32 = 2;

#[macroquad::main("Voxel World")]
async fn main() {
    let position = vec3(0.0, 0.0, 10.0);

    let mut world = World::new("test_world");
    let mut renderer = Renderer::new().await;
    let mut camera_controller = CameraController::new(position);

    //world.set(Location::new(10, 0, 10).into(), model::voxel::Voxel::Stone);
    //world.set(Location::new(10, 1, 10).into(), model::voxel::Voxel::Stone);
    //world.set(Location::new(9, 0, 10).into(), model::voxel::Voxel::Stone);
    //world.set(Location::new(10, 0, 9).into(), model::voxel::Voxel::Stone);
    let area = Location::new(10, 0, 10).into();
    world.unload_area(area);
    world.load_area(area);
    renderer.load_full_area(&mut world, area);
    camera_controller.set_focus(true);
    loop {
        let delta = get_frame_time();
        clear_background(BEIGE);

        camera_controller.update_look(delta);
        if move_forward() {
            camera_controller.move_forward(10.0, delta);
        }
        if move_back() {
            camera_controller.move_forward(-10.0, delta);
        }
        if move_left() {
            camera_controller.move_right(-10.0, delta);
        }
        if move_right() {
            camera_controller.move_right(10.0, delta);
        }
        if enter_focus() {
            camera_controller.set_focus(true);
        }
        if exit_focus() {
            camera_controller.set_focus(false);
        }

        let camera = camera_controller.create_camera();
        set_camera(&camera);
        let camera_location = camera_controller.get_camera_voxel_location();
        renderer.update_loaded_areas(
            &mut world,
            &get_render_zone(camera_location.into(), RENDER_SIZE),
        );
        world.retain_areas(&get_load_zone(camera_location.into(), RENDER_SIZE));

        renderer.render_voxels();

        next_frame().await;
    }
}
