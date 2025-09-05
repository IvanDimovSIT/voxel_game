use bincode::{Decode, Encode};

use crate::model::voxel::{MAX_VOXEL_VARIANTS, Voxel};

pub const MAX_ITEMS_PER_SLOT: u8 = 100;

/// stores the quantity of all inventory items
#[derive(Debug, Clone)]
pub struct AvailableItems {
    counts: Box<[u32; MAX_VOXEL_VARIANTS]>,
}
impl AvailableItems {
    pub fn new_empty() -> Self {
        Self {
            counts: Box::new([0; MAX_VOXEL_VARIANTS]),
        }
    }

    pub fn add(&mut self, voxel: Voxel, count: impl Into<u32>) {
        self.counts[voxel.index()] += count.into();
    }

    pub fn get(&self, voxel: Voxel) -> u32 {
        self.counts[voxel.index()]
    }
}

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

    pub fn new() -> Self {
        let mut inventory = Self::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 50);
        inventory.selected[1] = Item::some(Voxel::Glass, 50);
        inventory.selected[2] = Item::some(Voxel::Trampoline, 50);
        inventory.selected[3] = Item::some(Voxel::Boards, 50);
        inventory.selected[4] = Item::some(Voxel::Cobblestone, 50);
        inventory.selected[5] = Item::some(Voxel::Lamp, 50);

        inventory
    }

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

    /// creates a table of all the voxels in the inventory and their count
    pub fn create_all_items_map(&self) -> AvailableItems {
        let mut map = AvailableItems::new_empty();
        for item in self.selected.iter().chain(self.items.iter()).flatten() {
            map.add(item.voxel, item.count);
        }

        map
    }

    /// unchecked operation, may not remove desired amount
    pub fn remove_item(&mut self, mut item: Item) {
        let iterator = self
            .items
            .iter_mut()
            .chain(self.selected.iter_mut())
            .filter(|i| i.is_some() && i.unwrap().voxel == item.voxel);

        for slot in iterator {
            let amount_to_remove = slot.unwrap().count.min(item.count);
            *slot = slot.map(|inner| Item {
                count: inner.count - amount_to_remove,
                ..inner
            });
            item.count -= amount_to_remove;
            if slot.unwrap().count == 0 {
                *slot = None;
            }
            if item.count == 0 {
                return;
            }
        }

        debug_assert_eq!(item.count, 0);
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
        Self {
            items: [None; Self::INVENTORY_SIZE],
            selected: [None; Self::SELECTED_SIZE],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_add_item_into_empty() {
        let mut inventory = Inventory::default();
        inventory.add_item(Item::new(Voxel::Brick, 10));

        assert!(inventory.selected[0].is_some());
        assert!(inventory.selected[0].unwrap().count == 10);
        assert!(inventory.selected[0].unwrap().voxel == Voxel::Brick);
    }

    #[test]
    pub fn test_add_item_into_same_stack() {
        let mut inventory = Inventory::default();
        inventory.selected[1] = Item::some(Voxel::Brick, 20);
        inventory.add_item(Item::new(Voxel::Brick, 10));

        assert!(inventory.selected[1].is_some());
        assert!(inventory.selected[1].unwrap().count == 30);
        assert!(inventory.selected[1].unwrap().voxel == Voxel::Brick);
    }

    #[test]
    pub fn test_add_item_into_prexisting() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Grass, 20);
        inventory.add_item(Item::new(Voxel::Brick, 10));

        assert!(inventory.selected[1].is_some());
        assert!(inventory.selected[1].unwrap().count == 10);
        assert!(inventory.selected[1].unwrap().voxel == Voxel::Brick);
    }

    #[test]
    pub fn test_add_item_into_inventory() {
        let mut inventory = Inventory::default();
        inventory
            .selected
            .iter_mut()
            .for_each(|slot| *slot = Item::some(Voxel::Grass, 100));
        inventory.add_item(Item::new(Voxel::Brick, 10));

        assert!(inventory.items[0].is_some());
        assert!(inventory.items[0].unwrap().count == 10);
        assert!(inventory.items[0].unwrap().voxel == Voxel::Brick);
    }

    #[test]
    pub fn test_add_item_partial_join() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 80);
        inventory.add_item(Item::new(Voxel::Brick, 30));

        assert!(inventory.selected[0].is_some());
        assert!(inventory.selected[0].unwrap().count == 100);
        assert!(inventory.selected[0].unwrap().voxel == Voxel::Brick);
        assert!(inventory.selected[1].is_some());
        assert!(inventory.selected[1].unwrap().count == 10);
        assert!(inventory.selected[1].unwrap().voxel == Voxel::Brick);
    }

    #[test]
    pub fn test_create_all_items_map() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 80);
        inventory.selected[1] = Item::some(Voxel::Brick, 40);
        inventory.items[0] = Item::some(Voxel::Sand, 80);
        inventory.items[3] = Item::some(Voxel::Stone, 100);
        inventory.items[4] = Item::some(Voxel::Brick, 10);
        let map = inventory.create_all_items_map();

        assert_eq!(map.get(Voxel::Brick), 130);
        assert_eq!(map.get(Voxel::Sand), 80);
        assert_eq!(map.get(Voxel::Stone), 100);
        let other_items_count: u32 = map
            .counts
            .iter()
            .enumerate()
            .filter(|(voxel_index, _)| {
                ![
                    Voxel::Brick.index(),
                    Voxel::Sand.index(),
                    Voxel::Stone.index(),
                ]
                .contains(voxel_index)
            })
            .map(|(_, count)| *count)
            .sum();
        assert_eq!(other_items_count, 0);
    }

    #[test]
    pub fn test_remove_item() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 60);
        inventory.selected[1] = Item::some(Voxel::Brick, 40);
        inventory.remove_item(Item::new(Voxel::Brick, 70));

        assert!(inventory.selected[0].is_none());
        assert!(inventory.selected[1].is_some());
        assert_eq!(inventory.selected[1].unwrap().count, 30);
    }

    #[test]
    #[should_panic]
    pub fn test_remove_item_insufficient_quantity() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 60);
        inventory.remove_item(Item::new(Voxel::Brick, 70));
    }

    #[test]
    #[should_panic]
    pub fn test_remove_item_not_found() {
        let mut inventory = Inventory::default();
        inventory.selected[0] = Item::some(Voxel::Brick, 60);
        inventory.remove_item(Item::new(Voxel::Stone, 10));
    }
}
