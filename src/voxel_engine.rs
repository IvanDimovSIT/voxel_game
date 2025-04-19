use macroquad::{
    camera::set_default_camera,
    color::BEIGE,
    math::{Vec3, vec3},
    prelude::{error, gl_use_default_material},
    window::{clear_background, next_frame, screen_height, screen_width},
};

use crate::{
    graphics::{
        debug_display::DebugDisplay,
        renderer::Renderer,
        ui_display::{draw_crosshair, draw_selected_voxel, VoxelSelector},
    },
    model::{player_info::PlayerInfo, voxel::Voxel, world::World},
    service::{
        camera_controller::CameraController, input::{self, *}, raycast::{cast_ray, RaycastResult}, render_zone::{get_load_zone, get_render_zone}, voxel_physics::VoxelSimulator, world_actions::{destroy_voxel, place_voxel}
    },
    utils::vector_to_location,
};

pub struct VoxelEngine {
    world: World,
    renderer: Renderer,
    player_info: PlayerInfo,
    debug_display: DebugDisplay,
    voxel_selector: VoxelSelector,
    voxel_simulator: VoxelSimulator,
    render_size: u32,
}
impl VoxelEngine {
    pub async fn new(world_name: impl Into<String>) -> Self {
        let mut player_info = PlayerInfo::new(vec3(0.0, 0.0, 20.0));

        player_info.camera_controller.set_focus(true);
        Self {
            world: World::new(world_name),
            renderer: Renderer::new().await,
            player_info,
            debug_display: DebugDisplay::new(),
            render_size: 7,
            voxel_selector: VoxelSelector::new(),
            voxel_simulator: VoxelSimulator::new(),
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
        match get_scroll_direction() {
            ScrollDirection::Up => self.voxel_selector.select_next(),
            ScrollDirection::Down => self.voxel_selector.select_prev(),
            ScrollDirection::None => {},
        }

        raycast_result
    }

    pub fn process_physics(&mut self, delta: f32) {
        const GRAVITY: f32 = 25.0;
        const MAX_FALL_SPEED: f32 = 60.0;

        self.player_info.velocity =
            (self.player_info.velocity + GRAVITY * delta).min(MAX_FALL_SPEED);

        self.player_info.camera_controller.set_position(
            self.player_info.camera_controller.get_position()
                + vec3(0.0, 0.0, self.player_info.velocity * delta),
        );
        self.voxel_simulator.simulate_falling(&mut self.world, &mut self.renderer, delta);
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
        self.voxel_simulator.draw(&camera);
        gl_use_default_material();
        if let RaycastResult::Hit {
            first_non_empty,
            last_empty: _,
            distance: _,
        } = raycast_result
        {
            draw_selected_voxel(first_non_empty, &camera);
        }
        set_default_camera();
        draw_crosshair(width, height);
        self.voxel_selector.draw(self.renderer.get_texture_manager());
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
                distance: _,
            } => {
                let has_placed = place_voxel(
                    last_empty,
                    self.voxel_selector.get_selected(),
                    self.player_info
                        .camera_controller
                        .get_camera_voxel_location(),
                    &mut self.world,
                    &mut self.renderer,
                    &self.voxel_simulator
                );
                if !has_placed {
                    return;
                } 
                self.voxel_simulator.update_voxels(&mut self.world, &mut self.renderer, last_empty);
            }
        }
    }

    fn try_destroy_voxel(&mut self, raycast_result: RaycastResult) {
        match raycast_result {
            RaycastResult::NoneHit => {}
            RaycastResult::Hit {
                first_non_empty,
                last_empty: _,
                distance: _,
            } => {
                let has_destroyed = destroy_voxel(first_non_empty, &mut self.world, &mut self.renderer);
                if !has_destroyed {
                    return;
                }
                self.voxel_simulator.update_voxels(&mut self.world, &mut self.renderer, first_non_empty);
            }
        }
    }

    fn try_jump(&mut self) {
        let bottom_voxel_position = self.player_info.camera_controller.get_bottom_position();
        let voxel = self
            .world
            .get(vector_to_location(bottom_voxel_position).into());
        if voxel != Voxel::None {
            self.player_info.velocity = self.player_info.jump_velocity;
        }
    }

    fn try_move<D, M>(&mut self, velocity: f32, delta: f32, get_displacement: D, move_fn: M)
    where
        D: Fn(&CameraController, f32, f32) -> Vec3,
        M: Fn(&mut CameraController, f32, f32),
    {
        let top_position = self.player_info.camera_controller.get_position();
        let bottom_position =
            self.player_info.camera_controller.get_bottom_position() - vec3(0.0, 0.0, 0.05);
        let displacement = get_displacement(&self.player_info.camera_controller, velocity, delta);
        let displacement_magnitude = displacement.length();

        let top_result = cast_ray(
            &mut self.world,
            top_position,
            top_position + displacement,
            displacement_magnitude,
        );
        let bottom_result = cast_ray(
            &mut self.world,
            bottom_position,
            bottom_position + displacement,
            displacement_magnitude,
        );

        if matches!(top_result, RaycastResult::NoneHit)
            && matches!(bottom_result, RaycastResult::NoneHit)
        {
            move_fn(&mut self.player_info.camera_controller, velocity, delta);
        } else {
            let top_displacement = match top_result {
                RaycastResult::NoneHit => displacement_magnitude,
                RaycastResult::Hit { distance, .. } => distance,
            };
            let bottom_displacement = match bottom_result {
                RaycastResult::NoneHit => displacement_magnitude,
                RaycastResult::Hit { distance, .. } => distance,
            };
            let new_displacement = top_displacement.min(bottom_displacement) * 0.95;

            if new_displacement.abs() <= 0.05 {
                return;
            }

            move_fn(
                &mut self.player_info.camera_controller,
                new_displacement,
                if velocity < 0.0 { -1.0 } else { 1.0 },
            );
        }
    }

    fn try_move_forward(&mut self, velocity: f32, delta: f32) {
        self.try_move(
            velocity,
            delta,
            |camera_controller, v, d| camera_controller.get_forward_displacement(v, d),
            |camera_controller, v, d| camera_controller.move_forward(v, d),
        );
    }

    fn try_move_right(&mut self, velocity: f32, delta: f32) {
        self.try_move(
            velocity,
            delta,
            |camera_controller, v, d| camera_controller.get_right_displacement(v, d),
            |camera_controller, v, d| camera_controller.move_right(v, d),
        );
    }

    fn process_collisions(&mut self) {
        let mut camera_top_position = self.player_info.camera_controller.get_position();
        let mut camera_bottom_position = self.player_info.camera_controller.get_bottom_position();

        let voxel_bottom = self
            .world
            .get(vector_to_location(camera_bottom_position).into());
        if voxel_bottom != Voxel::None {
            let offset = camera_bottom_position.z - (camera_bottom_position.z.floor() + 0.5);
            camera_top_position -= vec3(0.0, 0.0, offset);
            camera_bottom_position -= vec3(0.0, 0.0, offset);
            self.player_info.velocity = 0.0;
        }

        let voxel_top = self
            .world
            .get(vector_to_location(camera_top_position).into());
        if voxel_top != Voxel::None {
            let offset = camera_top_position.z - (camera_top_position.z.floor() + 0.5);
            camera_top_position -= vec3(0.0, 0.0, offset);
            camera_bottom_position -= vec3(0.0, 0.0, offset);
            self.player_info.velocity = 0.0;
        }

        if self
            .world
            .get(vector_to_location(camera_bottom_position - vec3(0.0, 0.0, 0.1)).into())
            != Voxel::None
        {
            camera_top_position -= vec3(0.0, 0.0, 1.0);
            self.player_info.velocity = 0.0;
            error!("Stuck, moving up");
        }

        self.player_info
            .camera_controller
            .set_position(camera_top_position);
    }
}
