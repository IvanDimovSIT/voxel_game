use std::rc::Rc;

use macroquad::{
    camera::{Camera3D, set_default_camera},
    miniquad::window::screen_size,
    prelude::gl_use_default_material,
    window::next_frame,
};

use crate::{
    GameState,
    graphics::{
        debug_display::{DebugDisplay, DebugInfo},
        height_map::HeightMap,
        rain_system::RainSystem,
        renderer::Renderer,
        screen_effects::draw_water_effect,
        sky::Sky,
        ui_display::{draw_crosshair, draw_selected_voxel},
        voxel_particle_system::VoxelParticleSystem,
        world_map::WorldMap,
    },
    interface::{
        game_menu::{
            crafting_menu::{CraftingMenuContext, CraftingMenuHandle},
            game_menu_context::{MenuSelection, MenuState, draw_main_menu, draw_options_menu},
            voxel_selection_menu::draw_voxel_selection_menu,
        },
        interface_context::InterfaceContext,
        tutorial_messages::{TutorialMessage, TutorialMessages},
    },
    model::{inventory::Item, player_info::PlayerInfo, user_settings::UserSettings, world::World},
    service::{
        active_zone::{
            get_load_zone, get_load_zone_on_world_load, get_render_zone,
            get_render_zone_on_world_load,
        },
        activity_timer::ActivityTimer,
        asset_manager::AssetManager,
        creatures::creature_manager::CreatureManager,
        input::{self, ScrollDirection, move_right},
        persistence::{
            player_persistence::save_player_info,
            user_settings_persistence::write_user_settings_blocking,
            world_metadata_persistence::{WorldMetadata, store_world_metadata},
        },
        physics::{
            player_physics::{
                process_collisions, push_player_up_if_stuck, try_jump, try_move, try_swim,
            },
            voxel_simulator::VoxelSimulator,
        },
        raycast::{RaycastResult, cast_ray},
        sound_manager::SoundId,
        world_actions::{
            destroy_voxel, initialise_world_systems, place_voxel, replace_voxel,
            update_player_in_water,
        },
        world_time::WorldTime,
    },
};

pub struct VoxelEngine {
    world: World,
    renderer: Renderer,
    player_info: PlayerInfo,
    debug_display: DebugDisplay,
    voxel_simulator: VoxelSimulator,
    voxel_particles: VoxelParticleSystem,
    creature_manager: CreatureManager,
    asset_manager: Rc<AssetManager>,
    user_settings: UserSettings,
    menu_state: MenuState,
    world_time: WorldTime,
    sky: Sky,
    height_map: HeightMap,
    world_map: WorldMap,
    tutorial_messages: TutorialMessages,
    rain_system: RainSystem,
}
impl VoxelEngine {
    pub fn new(
        world_name: impl Into<String>,
        asset_manager: Rc<AssetManager>,
        user_settings: UserSettings,
    ) -> Self {
        let world_systems = initialise_world_systems(world_name, asset_manager.clone());

        Self {
            world: world_systems.world,
            renderer: world_systems.renderer,
            player_info: world_systems.player_info,
            debug_display: DebugDisplay::new(),
            user_settings,
            voxel_simulator: world_systems.voxel_simulator,
            asset_manager,
            menu_state: MenuState::Hidden,
            world_time: world_systems.world_time,
            sky: world_systems.sky,
            height_map: HeightMap::new(),
            voxel_particles: VoxelParticleSystem::new(),
            creature_manager: world_systems.creature_manager,
            world_map: WorldMap::new(),
            tutorial_messages: world_systems.tutorial_messages,
            rain_system: world_systems.rain_system,
        }
    }

    /// loads the world upon entering
    pub fn load_world(&mut self) {
        let camera_location = self
            .player_info
            .camera_controller
            .get_camera_voxel_location();

        let load_zone = get_load_zone_on_world_load(
            camera_location.into(),
            self.user_settings.get_render_distance(),
        );
        let render_zone = get_render_zone_on_world_load(
            camera_location.into(),
            self.user_settings.get_render_distance(),
        );
        self.world.load_all_blocking(&load_zone);
        self.renderer
            .load_all_blocking(&mut self.world, &render_zone);
        self.tutorial_messages.show(TutorialMessage::Initial);
    }

    fn check_change_render_distance(&mut self) {
        if input::decrease_render_distance() {
            let _changed = self.user_settings.decrease_render_distance();
        } else if input::increase_render_distance() {
            let _changed = self.user_settings.increase_render_distance();
        }
    }

    /// processes the player inputs and returns the looked at voxel from the camera
    pub fn process_input(&mut self, delta: f32) -> RaycastResult {
        self.manage_menu_state();
        self.check_change_render_distance();

        let raycast_result = self.process_mouse_input(delta);
        if self.menu_state.is_in_menu() {
            return raycast_result;
        }
        if input::is_show_map() {
            self.tutorial_messages.show(TutorialMessage::Map);
            self.world_map.active = !self.world_map.active;
        }
        if self.world_map.active {
            self.process_map_input(delta);

            return raycast_result;
        }

        if input::is_enter_inventory() {
            self.player_info.camera_controller.set_focus(false);
            self.menu_state = MenuState::ItemSelection {
                currently_selected_item: None,
            };
        } else if input::is_enter_crafting() {
            self.player_info.camera_controller.set_focus(false);
            self.menu_state =
                MenuState::Crafting(CraftingMenuContext::new(&self.player_info.inventory));
        }

        if input::is_start_place_voxel(&self.player_info.camera_controller) {
            self.try_place_voxel(raycast_result);
        } else if input::is_place_voxel(&self.player_info.camera_controller) {
            self.continue_world_action_progress(
                delta,
                raycast_result,
                |ve| &mut ve.player_info.place_progress,
                |ve, res| ve.try_place_voxel(res),
            );
        } else {
            self.player_info.place_progress.reset();
        }

        if input::is_start_destroy_voxel(&self.player_info.camera_controller) {
            self.try_destroy_voxel(raycast_result);
        } else if input::is_destroy_voxel(&self.player_info.camera_controller) {
            self.continue_world_action_progress(
                delta,
                raycast_result,
                |ve| &mut ve.player_info.destroy_progress,
                |ve, res| ve.try_destroy_voxel(res),
            );
        } else {
            self.player_info.destroy_progress.reset();
        }

        if input::is_start_replace_voxel(&self.player_info.camera_controller) {
            self.try_replace_voxel(raycast_result);
        } else if input::is_replace_voxel(&self.player_info.camera_controller) {
            self.continue_world_action_progress(
                delta,
                raycast_result,
                |ve| &mut ve.player_info.replace_progress,
                |ve, res| ve.try_replace_voxel(res),
            );
        } else {
            self.player_info.replace_progress.reset();
        }

        if input::jump() {
            try_jump(&mut self.player_info, &mut self.world);
        }
        if input::swim() {
            try_swim(&mut self.player_info, delta);
        }
        if input::move_forward() {
            self.try_move_forward(self.player_info.move_speed, delta);
        }
        if input::move_back() {
            self.try_move_forward(-self.player_info.move_speed, delta);
        }
        if input::move_left() {
            self.try_move_right(-self.player_info.move_speed, delta);
        }
        if input::move_right() {
            self.try_move_right(self.player_info.move_speed, delta);
        }
        if input::toggle_debug() {
            self.debug_display.toggle_display();
        }
        if let Some(number) = input::get_number_key() {
            self.player_info
                .voxel_selector
                .set_selected(number.wrapping_sub(1) as usize);
        }
        match input::get_scroll_direction() {
            ScrollDirection::Up => self.player_info.voxel_selector.select_next(),
            ScrollDirection::Down => self.player_info.voxel_selector.select_prev(),
            ScrollDirection::None => {}
        }

        raycast_result
    }

    fn process_map_input(&mut self, delta: f32) {
        match input::get_scroll_direction() {
            ScrollDirection::Up => self.world_map.decrease_zoom(delta),
            ScrollDirection::Down => self.world_map.increase_zoom(delta),
            ScrollDirection::None => {}
        }

        if input::move_forward() {
            self.world_map.increase_up_down_angle(delta);
        } else if input::move_back() {
            self.world_map.decrease_up_down_angle(delta);
        }
        if input::move_left() {
            self.world_map.increase_left_right_angle(delta);
        } else if move_right() {
            self.world_map.decrease_left_right_angle(delta);
        }
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
        if input::exit_focus() && !self.menu_state.is_in_menu() {
            self.menu_state = MenuState::Main;
            self.player_info.camera_controller.set_focus(false);
        } else if input::exit_focus() {
            self.menu_state = MenuState::Hidden;
            self.player_info.camera_controller.set_focus(true);
        }
    }

    /// updates time dependent processes
    pub fn update_processes(&mut self, delta: f32) {
        self.tutorial_messages.update(delta);

        if self.menu_state.is_in_menu() || self.world_map.active {
            return;
        }

        self.rain_system.update(
            delta,
            &self.player_info,
            &mut self.world,
            &self.user_settings,
        );
        self.world_time.update(delta);
        self.sky.update(delta);
        self.process_physics(delta);
        self.voxel_particles.update(delta);
        self.creature_manager.update(
            delta,
            &self.asset_manager.mesh_manager,
            &self.player_info,
            &mut self.world,
            &self.user_settings,
        );
        update_player_in_water(&mut self.player_info, &mut self.world);
    }

    /// process falling and collisions
    fn process_physics(&mut self, delta: f32) {
        let collision_type = process_collisions(&mut self.player_info, &mut self.world, delta);
        self.asset_manager
            .sound_manager
            .play_sound_for_collision(collision_type, &self.user_settings);

        self.voxel_particles.add_particles_for_collision(
            &self.player_info,
            collision_type,
            self.renderer.get_mesh_generator(),
        );

        push_player_up_if_stuck(&mut self.player_info, &mut self.world);
        self.voxel_simulator
            .update(&mut self.world, &mut self.renderer, delta);
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
        let (width, height) = screen_size();
        let camera = self.create_3d_camera();
        self.draw_background(&camera);

        // set 3D camera and voxel shader
        let visible_areas = self.renderer.set_voxel_shader_and_find_visible_areas(
            &camera,
            self.world_time
                .get_light_level(self.rain_system.get_light_level_modifier()),
            &self.user_settings,
            &self.world,
            &mut self.height_map,
            self.world_map.active,
        );
        let creatures_drawn = self.creature_manager.draw(&camera, &self.user_settings);
        self.voxel_particles.draw();
        self.voxel_simulator.draw(&camera);
        if !self.world_map.active {
            self.rain_system.draw_rain(&camera);
        }
        let rendered = self.renderer.render_voxels(
            &camera,
            &self.player_info,
            &self.user_settings,
            &visible_areas,
            self.world_map.active,
        );
        if !self.world_map.active {
            self.rain_system.draw_lightning(&camera);
        }

        // draw ui elements over 3D scene
        let menu_result = self.draw_ui_layer(
            width,
            height,
            &camera,
            raycast_result,
            rendered,
            creatures_drawn,
        );

        next_frame().await;
        menu_result
    }

    fn draw_background(&self, camera: &Camera3D) {
        if self.world_map.active {
            self.world_map.draw_background();
        } else {
            self.sky
                .draw_sky(&self.world_time, &self.rain_system, camera);
        }
    }

    fn create_3d_camera(&self) -> Camera3D {
        if self.world_map.active {
            self.world_map.create_map_camera(&self.player_info)
        } else {
            self.player_info.camera_controller.create_camera()
        }
    }

    /// draws the ui layer over the 3d scene
    fn draw_ui_layer(
        &mut self,
        width: f32,
        height: f32,
        camera: &Camera3D,
        raycast_result: RaycastResult,
        rendered: (usize, usize),
        creatures_drawn: u32,
    ) -> Option<GameState> {
        gl_use_default_material();
        if !self.world_map.active {
            self.draw_in_game_ui_elements(
                width,
                height,
                camera,
                raycast_result,
                rendered,
                creatures_drawn,
            );
        } else {
            set_default_camera();
            self.tutorial_messages.draw(height, &self.asset_manager);
        }

        self.process_menu()
    }

    /// draws the ui elements for the normal first person view
    fn draw_in_game_ui_elements(
        &self,
        width: f32,
        height: f32,
        camera: &Camera3D,
        raycast_result: RaycastResult,
        rendered: (usize, usize),
        creatures_drawn: u32,
    ) {
        if let RaycastResult::Hit {
            first_non_empty,
            last_empty: _,
        } = raycast_result
        {
            draw_selected_voxel(first_non_empty, camera);
        }
        self.debug_display
            .draw_area_border(&self.player_info.camera_controller);
        self.debug_display
            .draw_creature_bounding_boxes(&self.creature_manager, camera);

        // set 2D camera and draw 2D elements
        set_default_camera();
        if self.player_info.is_in_water {
            draw_water_effect(width, height, &self.asset_manager.texture_manager);
        }
        draw_crosshair(width, height);
        self.tutorial_messages.draw(height, &self.asset_manager);
        self.player_info
            .voxel_selector
            .draw(&self.player_info.inventory.selected, &self.asset_manager);

        let debug_info = DebugInfo {
            world: &self.world,
            renderer: &self.renderer,
            camera,
            rendered_areas_faces: rendered,
            creature_manager: &self.creature_manager,
            rendered_creatures: creatures_drawn,
        };
        self.debug_display
            .draw_debug_display(debug_info, &self.asset_manager.font);
    }

    /// returns the new game context only if changed
    fn process_menu(&mut self) -> Option<GameState> {
        match self.menu_state.clone() {
            MenuState::Hidden => None,
            MenuState::Main => self.process_main_menu(),
            MenuState::Options => self.process_options_menu(),
            MenuState::ItemSelection {
                currently_selected_item,
            } => self.process_voxel_selection_menu(currently_selected_item),
            MenuState::Crafting(handle) => self.process_crafting_menu(handle),
        }
    }

    fn process_crafting_menu(
        &mut self,
        crafting_menu_handle: CraftingMenuHandle,
    ) -> Option<GameState> {
        let menu_selection = crafting_menu_handle.borrow_mut().draw_menu(
            &mut self.player_info.inventory,
            &self.asset_manager,
            &self.user_settings,
        );

        self.handle_menu_selection(menu_selection)
    }

    fn process_voxel_selection_menu(
        &mut self,
        currently_selected_item: Option<Item>,
    ) -> Option<GameState> {
        let (selected_item, menu_selection) = draw_voxel_selection_menu(
            &self.asset_manager,
            &mut self.player_info,
            currently_selected_item,
        );
        if let MenuState::ItemSelection {
            currently_selected_item: _,
        } = self.menu_state
        {
            self.menu_state = MenuState::ItemSelection {
                currently_selected_item: selected_item,
            }
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
            &self.asset_manager,
            &mut self.user_settings,
            change_render_distance_callback,
        );
        self.handle_menu_selection(selection)
    }

    fn process_main_menu(&mut self) -> Option<GameState> {
        let selection = draw_main_menu(&self.asset_manager, &self.user_settings);
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
                    self.asset_manager.clone(),
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
                let selected_index = self.player_info.voxel_selector.get_selected_index();
                let selected_item = self.player_info.inventory.selected[selected_index];
                if selected_item.is_none() {
                    return;
                }

                let has_placed = place_voxel(
                    last_empty,
                    selected_item.unwrap().voxel,
                    &self.player_info,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                    &self.creature_manager,
                );
                if !has_placed {
                    return;
                }
                self.player_info
                    .inventory
                    .reduce_selected_at(selected_index);

                self.asset_manager
                    .sound_manager
                    .play_sound(SoundId::Place, &self.user_settings);

                self.tutorial_messages.show(TutorialMessage::Replacing);
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
                let maybe_destroyed = destroy_voxel(
                    first_non_empty,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                    &mut self.voxel_particles,
                );
                if let Some(destroyed) = maybe_destroyed {
                    self.player_info.inventory.add_item(Item::new(destroyed, 1));
                    self.asset_manager
                        .sound_manager
                        .play_sound(SoundId::Destroy, &self.user_settings);

                    self.tutorial_messages.show(TutorialMessage::Destroy);
                    if self.player_info.inventory.is_hotbar_full() {
                        self.tutorial_messages.show(TutorialMessage::Inventory);
                    }
                }
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
                let index = self.player_info.voxel_selector.get_selected_index();
                let selected_item = self.player_info.inventory.selected[index];
                if selected_item.is_none() {
                    return;
                }

                let maybe_replaced = replace_voxel(
                    first_non_empty,
                    selected_item.unwrap().voxel,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.voxel_simulator,
                );
                if let Some(replaced_voxel) = maybe_replaced {
                    self.player_info.inventory.reduce_selected_at(index);
                    self.player_info
                        .inventory
                        .add_item(Item::new(replaced_voxel, 1));
                    self.asset_manager
                        .sound_manager
                        .play_sound(SoundId::Destroy, &self.user_settings);
                }
            }
        }
    }

    /// performs a world action (place, destroy, break) based on an activity timer
    fn continue_world_action_progress<G, A>(
        &mut self,
        delta: f32,
        raycast_result: RaycastResult,
        get_activity_timer: G,
        world_action: A,
    ) where
        G: FnOnce(&mut VoxelEngine) -> &mut ActivityTimer,
        A: FnOnce(&mut VoxelEngine, RaycastResult),
    {
        match raycast_result {
            RaycastResult::NoneHit => {
                get_activity_timer(self).reset();
            }
            RaycastResult::Hit {
                first_non_empty: _,
                last_empty: _,
            } => {
                if get_activity_timer(self).tick(delta) {
                    world_action(self, raycast_result);
                }
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
        let world_metadata = WorldMetadata::new(
            &self.world_time,
            &self.voxel_simulator,
            &self.creature_manager,
            &self.sky,
            &self.tutorial_messages,
            &self.rain_system,
        );
        store_world_metadata(self.world.get_world_name(), world_metadata);
        write_user_settings_blocking(&self.user_settings);
        self.world.save_all_blocking();
    }
}
