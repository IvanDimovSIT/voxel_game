use graphics::{debug_display::DebugDisplay, renderer::Renderer};
use macroquad::{
    camera::set_camera,
    color::BEIGE,
    math::vec3,
    time::get_frame_time,
    window::{clear_background, next_frame},
};
use model::{location::Location, world::World};
use service::{
    camera_controller::CameraController,
    input::{
        enter_focus, exit_focus, is_destroy_voxel, is_place_voxel, move_back, move_forward,
        move_left, move_right, toggle_debug,
    },
    raycast::cast_ray,
    render_zone::{get_load_zone, get_render_zone},
    world_actions::{destroy_voxel, place_voxel},
};

mod graphics;
mod model;
mod service;
mod utils;

const RENDER_SIZE: u32 = 2;
const VOXEL_REACH: f32 = 7.0;

#[macroquad::main("Voxel World")]
async fn main() {
    let position = vec3(0.0, 0.0, 10.0);

    let mut world = World::new("test_world");
    let mut renderer = Renderer::new().await;
    let mut camera_controller = CameraController::new(position);
    let mut debug_display = DebugDisplay::new();

    let area = Location::new(10, 0, 10).into();
    world.unload_area(area);
    world.load_area(area);
    renderer.load_full_area(&mut world, area);
    camera_controller.set_focus(true);
    loop {
        let delta = get_frame_time();
        clear_background(BEIGE);

        camera_controller.update_look(delta);
        let camera = camera_controller.create_camera();
        if is_place_voxel(&camera_controller) {
            let result = cast_ray(&mut world, camera.position, camera.target, VOXEL_REACH);
            match result {
                service::raycast::RaycastResult::NoneHit => {}
                service::raycast::RaycastResult::Hit {
                    first_non_empty: _,
                    last_empty,
                } => {
                    let _ = place_voxel(
                        last_empty,
                        model::voxel::Voxel::Stone,
                        camera_controller.get_camera_voxel_location(),
                        &mut world,
                        &mut renderer,
                    );
                }
            }
        }
        if is_destroy_voxel(&camera_controller) {
            let result = cast_ray(&mut world, camera.position, camera.target, VOXEL_REACH);
            match result {
                service::raycast::RaycastResult::NoneHit => {}
                service::raycast::RaycastResult::Hit {
                    first_non_empty,
                    last_empty: _,
                } => {
                    let _ = destroy_voxel(first_non_empty, &mut world, &mut renderer);
                }
            }
        }

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
        if toggle_debug() {
            debug_display.toggle_display();
        }

        let camera_location = camera_controller.get_camera_voxel_location();
        renderer.update_loaded_areas(
            &mut world,
            &get_render_zone(camera_location.into(), RENDER_SIZE),
        );
        world.retain_areas(&get_load_zone(camera_location.into(), RENDER_SIZE));

        let rendered = renderer.render_voxels(&camera);
        debug_display.draw_debug_display(&world, &renderer, &camera, rendered);

        next_frame().await;
    }
}
