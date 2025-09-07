use std::{collections::HashSet, mem};

use bincode::{Decode, Encode};
use macroquad::prelude::error;

use crate::{
    graphics::renderer::Renderer,
    model::{area::AREA_HEIGHT, location::InternalLocation, voxel::Voxel, world::World},
    utils::StackVec,
};

const WATER_SPEED: f32 = 1.0;

const LOWEST_WATER: Voxel = Voxel::Water4;

#[derive(Debug, Clone, Encode, Decode)]
pub struct WaterSimulator {
    check_locations: HashSet<InternalLocation>,
    delta: f32,
}
impl WaterSimulator {
    pub fn new() -> Self {
        Self {
            check_locations: HashSet::new(),
            delta: 0.0,
        }
    }

    pub fn update(&mut self, world: &mut World, renderer: &mut Renderer, delta: f32) {
        self.delta += delta;
        if self.delta < WATER_SPEED {
            return;
        }

        self.delta -= WATER_SPEED;
        self.simulate_voxels(world, renderer);
    }

    pub fn location_updated(&mut self, location: impl Into<InternalLocation>) {
        let internal_location: InternalLocation = location.into();

        self.check_locations.insert(internal_location);
        self.check_locations.insert(internal_location.offset_x(1));
        self.check_locations.insert(internal_location.offset_x(-1));
        self.check_locations.insert(internal_location.offset_y(1));
        self.check_locations.insert(internal_location.offset_y(-1));

        if internal_location.z - 1 < AREA_HEIGHT {
            self.check_locations.insert(internal_location.offset_z(-1));
        }

        if internal_location.z > 0 {
            self.check_locations.insert(internal_location.offset_z(1));
        }
    }

    fn simulate_voxels(&mut self, world: &mut World, renderer: &mut Renderer) {
        let locations_to_check = mem::take(&mut self.check_locations);
        for location in locations_to_check {
            let voxel = world.get(location);
            if !Voxel::WATER.contains(&voxel) {
                continue;
            }
            let mut bordering = StackVec::new();
            let down = Self::get_bordering(location, world, &mut bordering);

            if self.check_flow_stopped(voxel, location, bordering.iter()) {
                world.set(location, Voxel::None);
                renderer.update_location(world, location);
                self.location_updated(location);
                continue;
            }

            if self.flow_down(location, down, world, renderer) {
                continue;
            }

            self.flow_sides(voxel, down, bordering, world, renderer);
        }
    }

    fn check_flow_stopped<'a>(
        &mut self,
        voxel: Voxel,
        current_location: InternalLocation,
        bordering: impl Iterator<Item = &'a (InternalLocation, Voxel)>,
    ) -> bool {
        debug_assert!(Voxel::WATER.contains(&voxel));
        if voxel == Voxel::WaterSource {
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
        if down_voxel.is_none() || Voxel::WATER.contains(&down_voxel.unwrap()) {
            return;
        }

        let sides = bordering.into_iter().take(4);
        let current_level = Self::get_water_level(voxel);

        for (side, side_voxel) in sides {
            let side_level = Self::get_water_level(side_voxel);
            if current_level <= side_level {
                continue;
            }

            let voxel_to_set = Self::reduce_water_level(voxel);
            world.set(side, voxel_to_set);
            renderer.update_location(world, side);
            self.check_locations.insert(side);
            println!("Flow sides for {voxel:?}");
        }
    }

    fn reduce_water_level(voxel: Voxel) -> Voxel {
        match voxel {
            Voxel::WaterSource => Voxel::Water1,
            Voxel::WaterDown => Voxel::Water1,
            Voxel::Water1 => Voxel::Water2,
            Voxel::Water2 => Voxel::Water3,
            Voxel::Water3 => Voxel::Water4,
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
            Voxel::WaterDown => 99,
            Voxel::Water1 => 98,
            Voxel::Water2 => 97,
            Voxel::Water3 => 96,
            Voxel::Water4 => 95,
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
            || (down_voxel != Voxel::WaterSource && Voxel::WATER.contains(&down_voxel));
        if !can_flow_down {
            return false;
        }

        world.set(down_location, Voxel::WaterDown);
        renderer.update_location(world, down_location);
        self.check_locations.insert(down_location);

        true
    }

    /// returns the bottom location, first 4 locations are always the sides
    fn get_bordering(
        location: InternalLocation,
        world: &mut World,
        vec: &mut StackVec<(InternalLocation, Voxel), 6>,
    ) -> Option<Voxel> {
        let offset_location = location.offset_x(1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_x(-1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_y(1);
        vec.push((offset_location, world.get(offset_location)));

        let offset_location = location.offset_y(-1);
        vec.push((offset_location, world.get(offset_location)));

        if location.z > 0 {
            let offset_location = location.offset_z(-1);
            vec.push((offset_location, world.get(offset_location)));
        }

        if location.z + 1 < AREA_HEIGHT {
            let offset_location = location.offset_z(1);
            let bottom_voxel = world.get(offset_location);
            vec.push((offset_location, bottom_voxel));

            Some(bottom_voxel)
        } else {
            None
        }
    }
}
