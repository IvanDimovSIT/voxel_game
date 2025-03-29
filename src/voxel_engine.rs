use macroquad::{
    camera::set_default_camera, color::BEIGE, math::vec3, prelude::gl_use_default_material, window::{clear_background, next_frame, screen_height, screen_width}
};

use crate::{
    graphics::{debug_display::DebugDisplay, renderer::Renderer, ui_display::{draw_crosshair, draw_selected_voxel}},
    model::{voxel::Voxel, world::World},
    service::{
        camera_controller::CameraController,
        input::*,
        raycast::{cast_ray, RaycastResult},
        render_zone::{get_load_zone, get_render_zone},
        world_actions::{destroy_voxel, place_voxel},
    },
};

struct PlayerInfo {
    camera_controller: CameraController,
    move_speed: f32,
    voxel_reach: f32,
}

pub struct VoxelEngine {
    world: World,
    renderer: Renderer,
    player_info: PlayerInfo,
    debug_display: DebugDisplay,
    render_size: u32,
}
impl VoxelEngine {
    pub async fn new(world_name: impl Into<String>) -> Self {
        let mut player_info = PlayerInfo {
            camera_controller: CameraController::new(vec3(0.0, 0.0, 20.0)),
            move_speed: 15.0,
            voxel_reach: 7.0,
        };

        player_info.camera_controller.set_focus(true);
        Self {
            world: World::new(world_name),
            renderer: Renderer::new().await,
            player_info,
            debug_display: DebugDisplay::new(),
            render_size: 7,
        }
    }

    /// returns the looked at voxel from the camera
    pub fn process_input(&mut self, delta: f32) -> RaycastResult {
        self.player_info.camera_controller.update_look(delta);
        let camera = self.player_info.camera_controller.create_camera();
        let raycast_result = cast_ray(
            &mut self.world,
            camera.position,
            camera.target,
            self.player_info.voxel_reach,
        );
        if is_place_voxel(&self.player_info.camera_controller) {
            match raycast_result {
                RaycastResult::NoneHit => {}
                RaycastResult::Hit {
                    first_non_empty: _,
                    last_empty,
                } => {
                    let _ = place_voxel(
                        last_empty,
                        Voxel::Stone,
                        self.player_info
                            .camera_controller
                            .get_camera_voxel_location(),
                        &mut self.world,
                        &mut self.renderer,
                    );
                }
            }
        }
        if is_destroy_voxel(&self.player_info.camera_controller) {
            match raycast_result {
                RaycastResult::NoneHit => {}
                RaycastResult::Hit {
                    first_non_empty,
                    last_empty: _,
                } => {
                    let _ = destroy_voxel(first_non_empty, &mut self.world, &mut self.renderer);
                }
            }
        }

        if move_forward() {
            self.player_info
                .camera_controller
                .move_forward(self.player_info.move_speed, delta);
        }
        if move_back() {
            self.player_info
                .camera_controller
                .move_forward(-self.player_info.move_speed, delta);
        }
        if move_left() {
            self.player_info
                .camera_controller
                .move_right(-self.player_info.move_speed, delta);
        }
        if move_right() {
            self.player_info
                .camera_controller
                .move_right(self.player_info.move_speed, delta);
        }
        if enter_focus() {
            self.player_info.camera_controller.set_focus(true);
        }
        if exit_focus() {
            self.player_info.camera_controller.set_focus(false);
        }
        if toggle_debug() {
            self.debug_display.toggle_display();
        }

        raycast_result
    }

    pub fn update_loaded_areas(&mut self) {
        let camera_location = self
            .player_info
            .camera_controller
            .get_camera_voxel_location();
        self.renderer.update_loaded_areas(
            &mut self.world,
            &get_render_zone(camera_location.into(), self.render_size),
        );
        self.world
            .retain_areas(&get_load_zone(camera_location.into(), self.render_size));
    }

    pub async fn draw_scene(&mut self, raycast_result: RaycastResult) {
        clear_background(BEIGE);

        let width = screen_width();
        let height = screen_height();
        let camera = self.player_info.camera_controller.create_camera();
        let rendered = self.renderer.render_voxels(&camera, self.render_size);
        gl_use_default_material();
        if let RaycastResult::Hit { first_non_empty, last_empty: _ } = raycast_result {
            draw_selected_voxel(first_non_empty);
        }
        set_default_camera();
        draw_crosshair(width, height);
        self.debug_display
            .draw_debug_display(&self.world, &self.renderer, &camera, rendered);
    
        next_frame().await;
    }
}
