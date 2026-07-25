use std::{collections::HashSet, mem};

use bincode::{Decode, Encode};
use macroquad::prelude::error;

use crate::{
    graphics::renderer::Renderer,
    model::{area::AREA_HEIGHT, location::InternalLocation, voxel::Voxel, world::World},
    service::activity_timer::ActivityTimer,
    utils::StackVec,
};

const WATER_ACTIVITY_DELAY_S: f32 = 0.3;

const LOWEST_WATER: Voxel = Voxel::Water6;

#[derive(Debug, Clone)]
pub struct WaterSimulator {
    check_locations: HashSet<InternalLocation>,
    activity_timer: ActivityTimer,
}
impl WaterSimulator {
    pub fn new() -> Self {
        Self {
            check_locations: HashSet::new(),
            activity_timer: ActivityTimer::new(0.0, WATER_ACTIVITY_DELAY_S),
        }
    }

    pub fn update(&mut self, world: &mut World, renderer: &mut Renderer, delta: f32) {
        if self.activity_timer.tick(delta) {
            self.simulate_voxels(world, renderer);
        }
    }

    pub fn location_updated(&mut self, location: impl Into<InternalLocation>) {
        let internal_location: InternalLocation = location.into();

        self.check_locations.insert(internal_location);
        self.check_locations.insert(internal_location.offset_x(1));
        self.check_locations.insert(internal_location.offset_x(-1));
        self.check_locations.insert(internal_location.offset_y(1));
        self.check_locations.insert(internal_location.offset_y(-1));

        if internal_location.z > 0 {
            self.check_locations.insert(internal_location.offset_z(-1));
        }

        if internal_location.z < AREA_HEIGHT - 1 {
            self.check_locations.insert(internal_location.offset_z(1));
        }
    }

    pub fn get_simulation_checks(&self) -> usize {
        self.check_locations.len()
    }

    pub fn create_dto(&self) -> WaterSimulatorDTO {
        WaterSimulatorDTO {
            check_locations: self.check_locations.clone(),
            check_delta: self.activity_timer.get_delta(),
        }
    }

    fn simulate_voxels(&mut self, world: &mut World, renderer: &mut Renderer) {
        let locations_to_check = mem::take(&mut self.check_locations);
        debug_assert!(self.check_locations.is_empty());
        for location in locations_to_check {
            let voxel = world.get(location);
            if !Voxel::WATER.contains(&voxel) {
                continue;
            }
            let mut bordering = StackVec::new();
            let (down_voxel, up_voxel) = Self::get_bordering(location, world, &mut bordering);

            self.update_water_source(voxel, location, up_voxel, world, renderer);
            if self.check_flow_stopped(voxel, location, bordering.iter()) {
                self.set_voxel(location, Voxel::None, world, renderer);
                debug_assert!(voxel != Voxel::None);
                continue;
            }

            if self.flow_down(location, down_voxel, world, renderer) {
                continue;
            }

            self.flow_sides(voxel, down_voxel, bordering, world, renderer);
        }
    }

    fn update_water_source(
        &mut self,
        voxel: Voxel,
        current_location: InternalLocation,
        up_voxel: Option<Voxel>,
        world: &mut World,
        renderer: &mut Renderer,
    ) {
        match voxel {
            Voxel::WaterSource => {
                if current_location.z == 0 {
                    return;
                }
                if let Some(up_voxel) = up_voxel
                    && Voxel::WATER.contains(&up_voxel)
                {
                    self.set_voxel(current_location, Voxel::WaterSourceFull, world, renderer);
                    debug_assert!(voxel != Voxel::WaterSourceFull);
                };
            }
            Voxel::WaterSourceFull => {
                if current_location.z == 0 {
                    self.set_voxel(current_location, Voxel::WaterSource, world, renderer);
                    return;
                }
                if let Some(up_voxel) = up_voxel
                    && !Voxel::WATER.contains(&up_voxel)
                {
                    self.set_voxel(current_location, Voxel::WaterSource, world, renderer);
                    debug_assert!(voxel != Voxel::WaterSource);
                };
            }
            _ => {}
        }
    }

    fn check_flow_stopped<'a>(
        &mut self,
        voxel: Voxel,
        current_location: InternalLocation,
        bordering: impl Iterator<Item = &'a (InternalLocation, Voxel)>,
    ) -> bool {
        debug_assert!(Voxel::WATER.contains(&voxel));
        if voxel == Voxel::WaterSource || voxel == Voxel::WaterSourceFull {
            return false;
        }
        let current_level = Self::get_water_level(voxel);
        for (bordering_location, bordering_voxel) in bordering {
            if !Voxel::WATER.contains(bordering_voxel) {
                continue;
            }
            if bordering_location.z > current_location.z {
                continue;
            }

            let bordering_level = Self::get_water_level(*bordering_voxel);
            if bordering_level > current_level
                || (voxel == Voxel::WaterDown && bordering_location.z != current_location.z)
            {
                return false;
            }
        }

        true
    }

    fn flow_sides(
        &mut self,
        voxel: Voxel,
        down_voxel: Option<Voxel>,
        bordering: StackVec<(InternalLocation, Voxel), 6>,
        world: &mut World,
        renderer: &mut Renderer,
    ) {
        if voxel == LOWEST_WATER {
            return;
        }
        if down_voxel.is_none() {
            return;
        }

        let down_voxel_some = down_voxel.unwrap();
        if Voxel::NON_SOURCE_WATER.contains(&down_voxel_some) {
            return;
        }

        let sides = bordering.into_iter().take(4);
        let voxel_to_set = Self::reduce_water_level(voxel);
        let new_level = Self::get_water_level(voxel_to_set);

        for (side, side_voxel) in sides {
            let side_level = Self::get_water_level(side_voxel);
            if new_level <= side_level {
                continue;
            }

            self.set_voxel(side, voxel_to_set, world, renderer);
            debug_assert!(voxel_to_set != side_voxel);
        }
    }

    /// sets the voxel in a location, updates the renderer and adds a location to check
    fn set_voxel(
        &mut self,
        location: InternalLocation,
        voxel: Voxel,
        world: &mut World,
        renderer: &mut Renderer,
    ) {
        world.set(location, voxel);
        renderer.update_location(world, location);
        self.location_updated(location);
    }

    fn reduce_water_level(voxel: Voxel) -> Voxel {
        match voxel {
            Voxel::WaterSource => Voxel::Water1,
            Voxel::WaterSourceFull => Voxel::Water1,
            Voxel::WaterDown => Voxel::Water1,
            Voxel::Water1 => Voxel::Water2,
            Voxel::Water2 => Voxel::Water3,
            Voxel::Water3 => Voxel::Water4,
            Voxel::Water4 => Voxel::Water5,
            Voxel::Water5 => Voxel::Water6,
            _ => {
                error!(
                    "Received non water voxel '{:?}' for 'reduce_water_level'",
                    voxel
                );
                Voxel::None
            }
        }
    }

    fn get_water_level(voxel: Voxel) -> i32 {
        match voxel {
            Voxel::WaterSource => 100,
            Voxel::WaterSourceFull => 100,
            Voxel::WaterDown => 99,
            Voxel::Water1 => 98,
            Voxel::Water2 => 97,
            Voxel::Water3 => 96,
            Voxel::Water4 => 95,
            Voxel::Water5 => 94,
            Voxel::Water6 => 93,
            Voxel::None => 0,
            _ => 1000,
        }
    }

    /// returns true if the flow was successful
    fn flow_down(
        &mut self,
        location: InternalLocation,
        down: Option<Voxel>,
        world: &mut World,
        renderer: &mut Renderer,
    ) -> bool {
        if location.z + 1 >= AREA_HEIGHT {
            return false;
        }
        let down_location = location.offset_z(1);
        let down_voxel = if let Some(some) = down {
            some
        } else {
            return false;
        };

        let can_flow_down = down_voxel == Voxel::None
            || (down_voxel != Voxel::WaterSource
                && down_voxel != Voxel::WaterSourceFull
                && Voxel::WATER.contains(&down_voxel));
        if !can_flow_down {
            return false;
        }

        world.set(down_location, Voxel::WaterDown);
        renderer.update_location(world, down_location);
        if down_voxel != Voxel::WaterDown {
            self.location_updated(down_location);
        }

        true
    }

    /// returns the (bottom location voxel, up location voxel), first 4 locations are always the sides
    fn get_bordering(
        location: InternalLocation,
        world: &mut World,
        vec: &mut StackVec<(InternalLocation, Voxel), 6>,
    ) -> (Option<Voxel>, Option<Voxel>) {
        let offset_location = location.offset_x(1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_x(-1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_y(1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_y(-1);
        vec.push((offset_location, world.get(offset_location)));

        let up_voxel = if location.z > 0 {
            let offset_location = location.offset_z(-1);
            vec.push((offset_location, world.get(offset_location)));

            Some(world.get(offset_location))
        } else {
            None
        };

        let down_voxel = if location.z + 1 < AREA_HEIGHT {
            let offset_location = location.offset_z(1);
            let bottom_voxel = world.get(offset_location);
            vec.push((offset_location, bottom_voxel));

            Some(bottom_voxel)
        } else {
            None
        };

        (down_voxel, up_voxel)
    }
}
impl From<WaterSimulatorDTO> for WaterSimulator {
    fn from(value: WaterSimulatorDTO) -> Self {
        Self {
            check_locations: value.check_locations,
            activity_timer: ActivityTimer::new(value.check_delta, WATER_ACTIVITY_DELAY_S),
        }
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct WaterSimulatorDTO {
    check_locations: HashSet<InternalLocation>,
    check_delta: f32,
}
