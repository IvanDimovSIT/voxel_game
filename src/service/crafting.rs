use crate::model::{
    inventory::{AvailableItems, Inventory, Item},
    voxel::Voxel,
};

impl Item {
    /// const constructor
    const fn new_c(voxel: Voxel, count: u8) -> Self {
        Self { voxel, count }
    }
}

const RECEPES: [CraftingRecipe; 5] = [
    CraftingRecipe::new1(Item::new_c(Voxel::Boards, 3), Item::new_c(Voxel::Wood, 1)),
    CraftingRecipe::new1(Item::new_c(Voxel::Glass, 1), Item::new_c(Voxel::Sand, 4)),
    CraftingRecipe::new1(
        Item::new_c(Voxel::Cobblestone, 1),
        Item::new_c(Voxel::Stone, 1),
    ),
    CraftingRecipe::new2(
        Item::new_c(Voxel::Brick, 4),
        Item::new_c(Voxel::Stone, 3),
        Item::new_c(Voxel::Clay, 1),
    ),
    CraftingRecipe::new1(
        Item::new_c(Voxel::WaterSource, 1),
        Item::new_c(Voxel::Snow, 2),
    ),
];

#[derive(Debug, Clone, Copy)]
pub struct CraftingRecipe {
    pub inputs: [Option<Item>; Self::MAX_INPUTS],
    pub output: Item,
}
impl CraftingRecipe {
    pub const MAX_INPUTS: usize = 3;

    const fn new1(output: Item, input: Item) -> Self {
        Self {
            inputs: [Some(input), None, None],
            output,
        }
    }

    const fn new2(output: Item, input1: Item, input2: Item) -> Self {
        Self {
            inputs: [Some(input1), Some(input2), None],
            output,
        }
    }

    pub fn get_inputs(&self) -> impl Iterator<Item = &Item> {
        self.inputs.iter().flatten()
    }
}

/// returns the craftable recipies and the number of times they can be crafted
pub fn find_craftable(available_items: &AvailableItems) -> Vec<(CraftingRecipe, u32)> {
    RECEPES
        .into_iter()
        .map(|recipe| (recipe, find_max_times_craftable(&recipe, available_items)))
        .filter(|(_, count)| *count > 0)
        .collect()
}

/// crafts the recipe a number of times
pub fn craft_recipe(recipe: &CraftingRecipe, inventory: &mut Inventory, craft_count: u8) {
    for input in recipe.get_inputs() {
        inventory.remove_item(Item::new(input.voxel, input.count * craft_count));
    }
    let output = Item::new(recipe.output.voxel, recipe.output.count * craft_count);
    inventory.add_item(output);
}

fn find_max_times_craftable(recipe: &CraftingRecipe, available_items: &AvailableItems) -> u32 {
    recipe
        .get_inputs()
        .map(|item| available_items.get(item.voxel) / item.count as u32)
        .min()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_craftable_some() {
        let mut available = AvailableItems::new_empty();
        available.add(Voxel::Wood, 10u32);
        available.add(Voxel::Stone, 10u32);
        available.add(Voxel::Clay, 10u32);

        let craftable = find_craftable(&available);
        assert_eq!(craftable.len(), 3);
        assert_eq!(craftable[0].0.output.voxel, Voxel::Boards);
        assert_eq!(craftable[0].1, 10);
        assert_eq!(craftable[1].0.output.voxel, Voxel::Cobblestone);
        assert_eq!(craftable[1].1, 10);
        assert_eq!(craftable[2].0.output.voxel, Voxel::Brick);
        assert_eq!(craftable[2].1, 3);
    }

    #[test]
    fn test_find_craftable_none() {
        let mut available = AvailableItems::new_empty();
        available.add(Voxel::Boards, 10u32);
        available.add(Voxel::Glass, 220u32);
        available.add(Voxel::Clay, 10u32);
        available.add(Voxel::Grass, 50u32);
        let craftable = find_craftable(&available);
        assert_eq!(craftable.len(), 0);
    }

    #[test]
    fn test_craft_recipe_once() {
        craft_recipe_with_count(1);
    }

    #[test]
    fn test_craft_recipe_five_times() {
        craft_recipe_with_count(5);
    }

    #[test]
    #[should_panic]
    fn test_craft_recipe_insufficient_inputs() {
        let mut inventory = Inventory::default();
        craft_recipe(&RECEPES[0], &mut inventory, 1);
    }

    fn craft_recipe_with_count(count: u8) {
        for recipe in RECEPES {
            let mut inventory = Inventory::default();
            for input in recipe.get_inputs() {
                inventory.add_item(Item {
                    count: input.count * count,
                    ..*input
                });
            }
            inventory.add_item(Item::new(recipe.output.voxel, 1));
            craft_recipe(&recipe, &mut inventory, count);

            let items = inventory.create_all_items_map();
            for input in recipe.get_inputs() {
                assert_eq!(items.get(input.voxel), 0);
            }

            assert_eq!(
                items.get(recipe.output.voxel),
                (recipe.output.count * count) as u32 + 1
            );
        }
    }
}
