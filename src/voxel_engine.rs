use std::{f32::consts::PI, rc::Rc};

use macroquad::{
    camera::set_default_camera, math::vec3, miniquad::window::screen_size,
    prelude::gl_use_default_material, window::next_frame,
};

use crate::{
    GameState,
    graphics::{
        debug_display::{DebugDisplay, DebugInfo},
        height_map::HeightMap,
        renderer::Renderer,
        screen_effects::draw_water_effect,
        sky::Sky,
        ui_display::{draw_crosshair, draw_selected_voxel},
        voxel_particle_system::VoxelParticleSystem,
    },
    interface::{
        game_menu::{
            crafting_menu::{CraftingMenuContext, CraftingMenuHandle},
            game_menu_context::{MenuSelection, MenuState, draw_main_menu, draw_options_menu},
            voxel_selection_menu::draw_voxel_selection_menu,
        },
        interface_context::InterfaceContext,
    },
    model::{inventory::Item, player_info::PlayerInfo, user_settings::UserSettings, world::World},
    service::{
        active_zone::{
            get_load_zone, get_load_zone_on_world_load, get_render_zone,
            get_render_zone_on_world_load,
        },
        asset_manager::AssetManager,
        creatures::creature_manager::CreatureManager,
        input::{self, ScrollDirection},
        persistence::{
            player_persistence::{load_player_info, save_player_info},
            user_settings_persistence::write_user_settings_blocking,
            world_metadata_persistence::{
                WorldMetadata, load_world_metadata, store_world_metadata,
            },
        },
        physics::{
            falling_voxel_simulator::FallingVoxelSimulator,
            player_physics::{
                process_collisions, push_player_up_if_stuck, try_jump, try_move, try_swim,
            },
            voxel_simulator::VoxelSimulator,
            water_simulator::WaterSimulator,
        },
        raycast::{RaycastResult, cast_ray},
        sound_manager::SoundId,
        world_actions::{
            destroy_voxel, place_voxel, put_player_on_ground, replace_voxel, update_player_in_water,
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
}
impl VoxelEngine {
    pub fn new(
        world_name: impl Into<String>,
        asset_manager: Rc<AssetManager>,
        user_settings: UserSettings,
    ) -> Self {
        let world_name = world_name.into();
        let (mut player_info, successful_load) = load_player_info(&world_name)
            .map(|info| (info, true))
            .unwrap_or_else(|| (PlayerInfo::new(vec3(0.0, 0.0, 0.0)), false));

        player_info.camera_controller.set_focus(true);
        let (world_time, simulated_voxels, water_simulator, creature_manager) =
            if let Some(world_metadata) = load_world_metadata(&world_name) {
                (
                    WorldTime::new(world_metadata.delta),
                    world_metadata.simulated_voxels,
                    world_metadata.water_simulator,
                    CreatureManager::from_dto(
                        world_metadata.creature_manager,
                        &asset_manager.mesh_manager,
                    ),
                )
            } else {
                (
                    WorldTime::new(PI * 0.5),
                    vec![],
                    WaterSimulator::new(),
                    CreatureManager::new(),
                )
            };

        let sky = Sky::new(&asset_manager.texture_manager);
        let renderer = Renderer::new(asset_manager.clone());
        let falling_voxel_simulator =
            FallingVoxelSimulator::new(simulated_voxels, renderer.get_mesh_generator());
        let voxel_simulator = VoxelSimulator::new(water_simulator, falling_voxel_simulator);

        let mut engine = Self {
            world: World::new(world_name),
            renderer,
            player_info,
            debug_display: DebugDisplay::new(),
            user_settings,
            voxel_simulator,
            asset_manager,
            menu_state: MenuState::Hidden,
            world_time,
            sky,
            height_map: HeightMap::new(),
            voxel_particles: VoxelParticleSystem::new(),
            creature_manager,
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
        if input::is_place_voxel(&self.player_info.camera_controller) {
            self.try_place_voxel(raycast_result);
        }
        if input::is_destroy_voxel(&self.player_info.camera_controller) {
            self.try_destroy_voxel(raycast_result);
        }
        if input::is_replace_voxel(&self.player_info.camera_controller) {
            self.try_replace_voxel(raycast_result);
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
        if self.menu_state.is_in_menu() {
            return;
        }
        self.world_time.update(delta);
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
        let camera = self.player_info.camera_controller.create_camera();
        self.sky.draw_sky(&self.world_time, &camera);

        // set 3D camera and voxel shader
        let visible_areas = self.renderer.set_voxel_shader_and_find_visible_areas(
            &camera,
            &self.world_time,
            &self.user_settings,
            &self.world,
            &mut self.height_map,
        );
        let creatures_drawn = self.creature_manager.draw(&camera, &self.user_settings);
        self.voxel_particles.draw();
        self.voxel_simulator.draw(&camera);
        let rendered = self.renderer.render_voxels(
            &camera,
            &self.player_info,
            &self.user_settings,
            &visible_areas,
        );

        // remove shader
        gl_use_default_material();
        if let RaycastResult::Hit {
            first_non_empty,
            last_empty: _,
        } = raycast_result
        {
            draw_selected_voxel(first_non_empty, &camera);
        }
        self.debug_display
            .draw_area_border(&self.player_info.camera_controller);

        // set 2D camera and draw 2D elements
        set_default_camera();
        if self.player_info.is_in_water {
            draw_water_effect(width, height, &self.asset_manager.texture_manager);
        }
        draw_crosshair(width, height);
        self.player_info
            .voxel_selector
            .draw(&self.player_info.inventory.selected, &self.asset_manager);

        let debug_info = DebugInfo {
            world: &self.world,
            renderer: &self.renderer,
            camera: &camera,
            rendered_areas_faces: rendered,
            creature_manager: &self.creature_manager,
            rendered_creatures: creatures_drawn,
        };
        self.debug_display
            .draw_debug_display(debug_info, &self.asset_manager.font);
        let menu_result = self.process_menu();

        next_frame().await;
        menu_result
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
        );
        store_world_metadata(world_metadata, self.world.get_world_name());
        write_user_settings_blocking(&self.user_settings);
        self.world.save_all_blocking();
    }
}
