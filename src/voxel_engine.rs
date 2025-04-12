use macroquad::{
    camera::set_default_camera,
    color::BEIGE,
    math::vec3,
    prelude::gl_use_default_material,
    window::{clear_background, next_frame, screen_height, screen_width},
};

use crate::{
    graphics::{
        debug_display::DebugDisplay,
        renderer::Renderer,
        ui_display::{draw_crosshair, draw_selected_voxel},
    },
    model::{voxel::Voxel, world::World},
    service::{
        camera_controller::CameraController,
        input::{self, *},
        raycast::{cast_ray, RaycastResult},
        render_zone::{get_load_zone, get_render_zone},
        world_actions::{destroy_voxel, place_voxel},
    }, utils::vector_to_location,
};

struct PlayerInfo {
    camera_controller: CameraController,
    move_speed: f32,
    voxel_reach: f32,
    velocity: f32,
    jump_velocity: f32
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
            move_speed: 10.0,
            voxel_reach: 7.0,
            velocity: 0.0,
            jump_velocity: -0.15,
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

    fn check_change_render_distance(&mut self) {
        const MIN_RENDER_DISTANCE: u32 = 3;
        const MAX_RENDER_DISTANCE: u32 = 14;
        if input::decrease_render_distance() && self.render_size > MIN_RENDER_DISTANCE {
            self.render_size -= 1;
        } else if input::increase_render_distance() && self.render_size < MAX_RENDER_DISTANCE {
            self.render_size += 1;
        }
    }

    /// returns the looked at voxel from the camera
    pub fn process_input(&mut self, delta: f32) -> RaycastResult {
        self.player_info.camera_controller.update_look(delta);
        let camera = self.player_info.camera_controller.create_camera();
        self.check_change_render_distance();
        let raycast_result = cast_ray(
            &mut self.world,
            camera.position,
            camera.target,
            self.player_info.voxel_reach,
        );
        if is_place_voxel(&self.player_info.camera_controller) {
            self.try_place_voxel(raycast_result);
        }
        if is_destroy_voxel(&self.player_info.camera_controller) {
            self.try_destroy_voxel(raycast_result);
        }
        if jump() {
            self.try_jump();
        }
        if move_forward() {
            self.try_move_forward(self.player_info.move_speed, delta);
        }
        if move_back() {
            self.try_move_forward(-self.player_info.move_speed, delta);
        }
        if move_left() {
            self.try_move_right(-self.player_info.move_speed, delta);
        }
        if move_right() {
            self.try_move_right(self.player_info.move_speed, delta);
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

    pub fn process_physics(&mut self, delta: f32) {
        const GRAVITY: f32 = 0.2;
        const MAX_FALL_SPEED: f32 = 2.5;

        self.player_info.velocity = (self.player_info.velocity + GRAVITY * delta).min(MAX_FALL_SPEED);

        self.player_info.camera_controller.set_position(
            self.player_info.camera_controller.get_position() + vec3(0.0, 0.0, self.player_info.velocity)
        );
        self.process_collisions();
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
        if let RaycastResult::Hit {
            first_non_empty,
            last_empty: _,
        } = raycast_result
        {
            draw_selected_voxel(first_non_empty, &camera);
        }
        set_default_camera();
        draw_crosshair(width, height);
        self.debug_display
            .draw_debug_display(&self.world, &self.renderer, &camera, rendered);

        next_frame().await;
    }

    fn try_place_voxel(&mut self, raycast_result: RaycastResult) {
        match raycast_result {
            RaycastResult::NoneHit => {}
            RaycastResult::Hit {
                first_non_empty: _,
                last_empty,
            } => {
                let _ = place_voxel(
                    last_empty,
                    Voxel::Leaves,
                    self.player_info
                        .camera_controller
                        .get_camera_voxel_location(),
                    &mut self.world,
                    &mut self.renderer,
                );
            }
        }
    }

    fn try_destroy_voxel(&mut self, raycast_result: RaycastResult) {
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

    fn try_jump(&mut self) {
        let bottom_voxel_position = self.player_info.camera_controller.get_bottom_position();
        let voxel = self.world.get(vector_to_location(bottom_voxel_position).into());
        if voxel != Voxel::None {
            self.player_info.velocity = self.player_info.jump_velocity;
        }
    }

    fn try_move_forward(&mut self, velocity: f32, delta: f32) {
        self.player_info
            .camera_controller
            .move_forward(velocity, delta);

        let top_position = self.player_info.camera_controller.get_position() - vec3(0.0, 0.0, 0.1);
        let bottom_position = self.player_info.camera_controller.get_bottom_position() - vec3(0.0, 0.0, 0.1);
        let top_voxel = self.world.get(vector_to_location(top_position).into());
        let bottom_voxel = self.world.get(vector_to_location(bottom_position).into());

        if top_voxel == Voxel::None && bottom_voxel == Voxel::None {
            return;
        }        

        self.player_info
            .camera_controller
            .move_forward(-velocity, delta);
    }

    fn try_move_right(&mut self, velocity: f32, delta: f32) {
        self.player_info
            .camera_controller
            .move_right(velocity, delta);

        let top_position = self.player_info.camera_controller.get_position() - vec3(0.0, 0.0, 0.1);
        let bottom_position = self.player_info.camera_controller.get_bottom_position() - vec3(0.0, 0.0, 0.1);
        let top_voxel = self.world.get(vector_to_location(top_position).into());
        let bottom_voxel = self.world.get(vector_to_location(bottom_position).into());

        if top_voxel == Voxel::None && bottom_voxel == Voxel::None {
            return;
        }        

        self.player_info
            .camera_controller
            .move_right(-velocity, delta);
    }
    
    fn process_collisions(&mut self) {
        let mut camera_top_position = self.player_info.camera_controller.get_position();
        let mut camera_bottom_position = self.player_info.camera_controller.get_bottom_position();
        
        let voxel_bottom = self.world.get(vector_to_location(camera_bottom_position).into());
        if voxel_bottom != Voxel::None {
            let offset = camera_bottom_position.z - (camera_bottom_position.z.floor() + 0.5);
            camera_top_position -= vec3(0.0, 0.0, offset);
            camera_bottom_position -= vec3(0.0, 0.0, offset);
            self.player_info.velocity = 0.0;
        }

        let voxel_top = self.world.get(vector_to_location(camera_top_position).into());
        if voxel_top != Voxel::None {
            let offset = camera_top_position.z - (camera_top_position.z.floor() + 0.5);
            camera_top_position -= vec3(0.0, 0.0, offset);
            camera_bottom_position -= vec3(0.0, 0.0, offset);
            self.player_info.velocity = 0.0;
        }

        self.player_info.camera_controller.set_position(camera_top_position);
    }
}
