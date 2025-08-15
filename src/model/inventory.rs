use bincode::{Decode, Encode};

use crate::model::voxel::Voxel;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct Item {
    pub voxel: Voxel,
    pub count: u32,
}
impl Item {
    pub fn some(voxel: Voxel, count: u32) -> Option<Item> {
        Some(Item { voxel, count })
    }
}

pub const INVENTORY_SIZE: usize = 40;
pub const SELECTED_SIZE: usize = 8;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Inventory {
    pub items: [Option<Item>; INVENTORY_SIZE],
    pub selected: [Option<Item>; SELECTED_SIZE],
}
impl Inventory {
    pub fn add_item(&mut self, item: Item) {
        for inventory_item in self.selected.iter_mut().chain(self.items.iter_mut()) {
            if let Some(found_item) = inventory_item {
                if found_item.voxel == item.voxel {
                    found_item.count += item.count;
                    return;
                }
            }
        }
        for inventory_item in self.selected.iter_mut().chain(self.items.iter_mut()) {
            if inventory_item.is_none() {
                *inventory_item = Some(item);
                return;
            }
        }
    }

    pub fn reduce_selected_at(&mut self, index: usize) {
        debug_assert!(self.selected[index].is_some());
        debug_assert!(self.selected[index].unwrap().count > 0);
        let mut item = self.selected[index].unwrap();
        item.count -= 1;
        self.selected[index] = if item.count == 0 { None } else { Some(item) }
    }
}
impl Default for Inventory {
    fn default() -> Self {
        let mut selected = [None; SELECTED_SIZE];
        selected[0] = Item::some(Voxel::Brick, 10);
        Self {
            items: [None; INVENTORY_SIZE],
            selected,
        }
    }
}
