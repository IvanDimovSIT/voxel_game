use std::{f32::consts::PI, rc::Rc};

use macroquad::{
    camera::set_default_camera,
    math::vec3,
    prelude::gl_use_default_material,
    window::{next_frame, screen_height, screen_width},
};

use crate::{
    GameState,
    graphics::{
        debug_display::DebugDisplay,
        max_height::calculate_max_height_around_location,
        renderer::Renderer,
        sky::draw_sky,
        texture_manager::TextureManager,
        ui_display::{draw_crosshair, draw_selected_voxel},
    },
    interface::{
        game_menu::{
            game_menu::{MenuSelection, MenuState, draw_main_menu, draw_options_menu},
            voxel_selection_menu::draw_voxel_selection_menu,
        },
        interface_context::InterfaceContext,
    },
    model::{player_info::PlayerInfo, user_settings::UserSettings, voxel::Voxel, world::World},
    service::{
        input::{self, *},
        persistence::{
            player_persistence::{load_player_info, save_player_info},
            user_settings_persistence::write_user_settings_blocking,
            world_metadata_persistence::{
                WorldMetadata, load_world_metadata, store_world_metadata,
            },
        },
        physics::{
            player_physics::{
                CollisionType, process_collisions, push_player_up_if_stuck, try_jump, try_move,
            },
            voxel_physics::VoxelSimulator,
        },
        raycast::{RaycastResult, cast_ray},
        render_zone::{get_load_zone, get_render_zone},
        sound_manager::{SoundId, SoundManager},
        world_actions::{destroy_voxel, place_voxel, put_player_on_ground, replace_voxel},
        world_time::WorldTime,
    },
};

pub struct VoxelEngine {
    world: World,
    renderer: Renderer,
    player_info: PlayerInfo,
    debug_display: DebugDisplay,
    voxel_simulator: VoxelSimulator,
    sound_manager: Rc<SoundManager>,
    user_settings: UserSettings,
    menu_state: MenuState,
    world_time: WorldTime,
}
impl VoxelEngine {
    pub fn new(
        world_name: impl Into<String>,
        texture_manager: Rc<TextureManager>,
        sound_manager: Rc<SoundManager>,
        user_settings: UserSettings,
    ) -> Self {
        let world_name = world_name.into();
        let (mut player_info, successful_load) = load_player_info(&world_name)
            .map(|x| (x, true))
            .unwrap_or_else(|| (PlayerInfo::new(vec3(0.0, 0.0, 0.0)), false));

        player_info.camera_controller.set_focus(true);
        let (world_time, simulated_voxels) =
            if let Some(world_metadata) = load_world_metadata(&world_name) {
                (
                    WorldTime::new(world_metadata.delta),
                    world_metadata.simulated_voxels,
                )
            } else {
                (WorldTime::new(PI * 0.5), vec![])
            };

        let renderer = Renderer::new(texture_manager);
        let voxel_simulator = VoxelSimulator::new(simulated_voxels, renderer.get_mesh_generator());

        let mut engine = Self {
            world: World::new(world_name),
            renderer,
            player_info,
            debug_display: DebugDisplay::new(),
            user_settings,
            voxel_simulator,
            sound_manager,
            menu_state: MenuState::Hidden,
            world_time,
        };

        if !successful_load {
            put_player_on_ground(&mut engine.player_info, &mut engine.world);
        }

        engine
    }

    /// loads the world upon entering
    pub fn load_world(&mut self) {
        let camera_location = self
            .player_info
            .camera_controller
            .get_camera_voxel_location();

        let load_zone = get_load_zone(
            camera_location.into(),
            self.user_settings.get_render_distance(),
        );
        let render_zone = get_render_zone(
            camera_location.into(),
            self.user_settings.get_render_distance(),
        );
        self.world.load_all_blocking(&load_zone);
        self.renderer
            .load_all_blocking(&mut self.world, &render_zone);
    }

    fn check_change_render_distance(&mut self) {
        if input::decrease_render_distance() {
            let _changed = self.user_settings.decrease_render_distance();
        } else if input::increase_render_distance() {
            let _changed = self.user_settings.increase_render_distance();
        }
    }

    /// processes and player inputs and returns the looked at voxel from the camera
    pub fn process_input(&mut self, delta: f32) -> RaycastResult {
        self.manage_menu_state();
        self.check_change_render_distance();

        let raycast_result = self.process_mouse_input(delta);
        if self.menu_state.is_in_menu() {
            return raycast_result;
        }

        if is_enter_inventory() {
            self.player_info.camera_controller.set_focus(false);
            self.menu_state = MenuState::VoxelSelection(None);
        }
        if is_place_voxel(&self.player_info.camera_controller) {
            self.try_place_voxel(raycast_result);
        }
        if is_destroy_voxel(&self.player_info.camera_controller) {
            self.try_destroy_voxel(raycast_result);
        }
        if is_replace_voxel(&self.player_info.camera_controller) {
            self.try_replace_voxel(raycast_result);
        }
        if jump() {
            try_jump(&mut self.player_info, &mut self.world);
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
        if toggle_debug() {
            self.debug_display.toggle_display();
        }
        match get_scroll_direction() {
            ScrollDirection::Up => self.player_info.voxel_selector.select_next(),
            ScrollDirection::Down => self.player_info.voxel_selector.select_prev(),
            ScrollDirection::None => {}
        }

        raycast_result
    }

    fn process_mouse_input(&mut self, delta: f32) -> RaycastResult {
        self.player_info.camera_controller.update_look(delta);
        let camera = self.player_info.camera_controller.create_camera();

        cast_ray(
            &mut self.world,
            camera.position,
            camera.target,
            self.player_info.voxel_reach,
        )
    }

    fn manage_menu_state(&mut self) {
        if exit_focus() && !self.menu_state.is_in_menu() {
            self.menu_state = MenuState::Main;
            self.player_info.camera_controller.set_focus(false);
        } else if exit_focus() {
            self.menu_state = MenuState::Hidden;
            self.player_info.camera_controller.set_focus(true);
        }
    }

    /// updates time dependent processes
    pub fn update_processes(&mut self, delta: f32) {
        if self.menu_state.is_in_menu() {
            return;
        }
        self.world_time.update(delta);
        self.process_physics(delta);
    }

    /// process falling and collisions
    fn process_physics(&mut self, delta: f32) {
        let collision_type = process_collisions(&mut self.player_info, &mut self.world, delta);
        if collision_type == CollisionType::Strong {
            self.sound_manager
                .play_sound(SoundId::Fall, &self.user_settings);
        }

        push_player_up_if_stuck(&mut self.player_info, &mut self.world);
        self.voxel_simulator
            .simulate_falling(&mut self.world, &mut self.renderer, delta);
    }

    /// updates the areas loaded in memory and unloads old areas
    pub fn update_loaded_areas(&mut self) {
        let camera_location = self
            .player_info
            .camera_controller
            .get_camera_voxel_location();
        let render_size = self.user_settings.get_render_distance();
        self.renderer
            .update_loaded_areas(&get_render_zone(camera_location.into(), render_size));
        self.renderer.load_areas_in_queue(&mut self.world);
        self.world
            .retain_areas(&get_load_zone(camera_location.into(), render_size));
    }

    /// draws the current frame, return the new context if changed
    pub async fn draw_scene(&mut self, raycast_result: RaycastResult) -> Option<GameState> {
        draw_sky(&self.world_time);

        let width = screen_width();
        let height = screen_height();
        let camera = self.player_info.camera_controller.create_camera();
        let rendered = self.renderer.render_voxels(
            &camera,
            self.user_settings.get_render_distance(),
            &self.world_time,
            calculate_max_height_around_location(
                &mut self.world,
                self.player_info
                    .camera_controller
                    .get_camera_voxel_location(),
                &self.user_settings,
            ),
            &self.user_settings,
        );
        self.voxel_simulator.draw(&camera);
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
        self.player_info
            .voxel_selector
            .draw(self.renderer.get_texture_manager());
        self.debug_display
            .draw_debug_display(&self.world, &self.renderer, &camera, rendered);
        let menu_result = self.process_menu();

        next_frame().await;
        menu_result
    }

    /// returns the new game context only if changed
    fn process_menu(&mut self) -> Option<GameState> {
        match self.menu_state {
            MenuState::Hidden => None,
            MenuState::Main => self.process_main_menu(),
            MenuState::Options => self.process_options_menu(),
            MenuState::VoxelSelection(voxel) => self.process_voxel_selection_menu(voxel),
        }
    }

    fn process_voxel_selection_menu(&mut self, selected_voxel: Option<Voxel>) -> Option<GameState> {
        let (selected_voxel, menu_selection) = draw_voxel_selection_menu(
            self.renderer.get_texture_manager(),
            &mut self.player_info,
            selected_voxel,
        );
        if let MenuState::VoxelSelection(_) = self.menu_state {
            self.menu_state = MenuState::VoxelSelection(selected_voxel)
        }

        self.handle_menu_selection(menu_selection)
    }

    fn process_options_menu(&mut self) -> Option<GameState> {
        let change_render_distance_callback = |settings: &UserSettings| {
            let render_size = settings.get_render_distance();
            self.renderer.load_all_blocking(
                &mut self.world,
                &get_render_zone(
                    self.player_info
                        .camera_controller
                        .get_camera_voxel_location()
                        .into(),
                    render_size,
                ),
            );
        };
        let selection = draw_options_menu(
            &self.sound_manager,
            &mut self.user_settings,
            change_render_distance_callback,
        );
        self.handle_menu_selection(selection)
    }

    fn process_main_menu(&mut self) -> Option<GameState> {
        let selection = draw_main_menu(&self.sound_manager, &self.user_settings);
        self.handle_menu_selection(selection)
    }

    fn handle_menu_selection(&mut self, selection: MenuSelection) -> Option<GameState> {
        match selection {
            MenuSelection::None => None,
            MenuSelection::BackToGame => {
                self.player_info.camera_controller.set_focus(true);
                self.menu_state = MenuState::Hidden;
                None
            }
            MenuSelection::ToWorldSelection => Some(GameState::Menu {
                context: Box::new(InterfaceContext::new_world_selection(
                    self.sound_manager.clone(),
                    self.renderer.get_texture_manager_copy(),
                    self.user_settings.clone(),
                )),
            }),
            MenuSelection::Exit => Some(GameState::Exit),
            MenuSelection::ToOptions => {
                self.menu_state = MenuState::Options;
                None
            }
            MenuSelection::ToMainMenu => {
                self.menu_state = MenuState::Main;
                None
            }
        }
    }

    fn try_place_voxel(&mut self, raycast_result: RaycastResult) {
        match raycast_result {
            RaycastResult::NoneHit => {}
            RaycastResult::Hit {
                first_non_empty: _,
                last_empty,
            } => {
                let selected_voxel = self.player_info.voxel_selector.get_selected();
                if selected_voxel.is_none() {
                    return;
                }

                let has_placed = place_voxel(
                    last_empty,
                    selected_voxel.unwrap(),
                    &self.player_info,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                );
                if !has_placed {
                    return;
                }

                self.sound_manager
                    .play_sound(SoundId::Place, &self.user_settings);
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
                let has_destroyed = destroy_voxel(
                    first_non_empty,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                );
                if !has_destroyed {
                    return;
                }

                self.sound_manager
                    .play_sound(SoundId::Destroy, &self.user_settings);
            }
        }
    }

    fn try_replace_voxel(&mut self, raycast_result: RaycastResult) {
        match raycast_result {
            RaycastResult::NoneHit => {}
            RaycastResult::Hit {
                first_non_empty,
                last_empty: _,
            } => {
                let selected_voxel = self.player_info.voxel_selector.get_selected();
                if selected_voxel.is_none() {
                    return;
                }

                let has_replaced = replace_voxel(
                    first_non_empty,
                    selected_voxel.unwrap(),
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                );
                if !has_replaced {
                    return;
                }

                self.sound_manager
                    .play_sound(SoundId::Destroy, &self.user_settings);
            }
        }
    }

    fn try_move_forward(&mut self, velocity: f32, delta: f32) {
        let displacement = self
            .player_info
            .camera_controller
            .get_forward_displacement(velocity, delta);
        try_move(&mut self.player_info, &mut self.world, displacement);
    }

    fn try_move_right(&mut self, velocity: f32, delta: f32) {
        let displacement = self
            .player_info
            .camera_controller
            .get_right_displacement(velocity, delta);
        try_move(&mut self.player_info, &mut self.world, displacement);
    }
}
impl Drop for VoxelEngine {
    fn drop(&mut self) {
        save_player_info(self.world.get_world_name(), &self.player_info);
        let world_metadata = WorldMetadata::new(&self.world_time, &self.voxel_simulator);
        store_world_metadata(world_metadata, self.world.get_world_name());
        write_user_settings_blocking(&self.user_settings);
        self.world.save_all_blocking();
    }
}
