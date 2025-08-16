use bincode::{Decode, Encode};

use crate::model::voxel::Voxel;

pub const MAX_ITEMS_PER_SLOT: u8 = 100;

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct Item {
    pub voxel: Voxel,
    pub count: u8,
}
impl Item {
    pub fn new(voxel: Voxel, count: u8) -> Item {
        debug_assert!(voxel != Voxel::None);
        debug_assert!(count > 0);
        debug_assert!(count <= MAX_ITEMS_PER_SLOT);
        Item { voxel, count }
    }

    pub fn some(voxel: Voxel, count: u8) -> Option<Item> {
        Some(Self::new(voxel, count))
    }
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Inventory {
    pub items: [Option<Item>; Self::INVENTORY_SIZE],
    pub selected: [Option<Item>; Self::SELECTED_SIZE],
}
impl Inventory {
    pub const INVENTORY_SIZE: usize = 40;
    pub const SELECTED_SIZE: usize = 8;

    pub fn add_item(&mut self, mut item: Item) {
        let items_iterator = self
            .selected
            .iter_mut()
            .chain(self.items.iter_mut())
            .flatten();

        for inventory_item in items_iterator {
            if inventory_item.voxel == item.voxel {
                let to_add = (MAX_ITEMS_PER_SLOT - inventory_item.count).min(item.count);
                inventory_item.count += to_add;
                item.count -= to_add;
                if item.count == 0 {
                    return;
                }
            }
        }
        if let Some(empty_slot) = self
            .selected
            .iter_mut()
            .chain(self.items.iter_mut())
            .find(|slot| slot.is_none())
        {
            *empty_slot = Some(item);
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
        let mut selected = [None; Self::SELECTED_SIZE];
        selected[0] = Item::some(Voxel::Brick, 50);
        selected[1] = Item::some(Voxel::Glass, 50);
        selected[2] = Item::some(Voxel::Trampoline, 50);
        selected[3] = Item::some(Voxel::Boards, 50);
        selected[4] = Item::some(Voxel::Cobblestone, 50);
        selected[5] = Item::some(Voxel::Lamp, 50);
        Self {
            items: [None; Self::INVENTORY_SIZE],
            selected,
        }
    }
}
