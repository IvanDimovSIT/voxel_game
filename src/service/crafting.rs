use std::collections::HashMap;

use crate::model::{inventory::Item, voxel::Voxel};

impl Item {
    /// const constructor
    const fn new_c(voxel: Voxel, count: u8) -> Self {
        Self { voxel, count }
    }
}

const RECEPES: [CraftingRecipe; 4] = [
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

pub fn find_craftable(available_items: &HashMap<Voxel, u32>) -> Vec<CraftingRecipe> {
    RECEPES
        .into_iter()
        .filter(|recipe| can_craft_recipe(recipe, available_items))
        .collect()
}

fn can_craft_recipe(recipe: &CraftingRecipe, available_items: &HashMap<Voxel, u32>) -> bool {
    recipe.inputs.into_iter().flatten().all(|input| {
        if let Some(available) = available_items.get(&input.voxel) {
            *available >= input.count as u32
        } else {
            false
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_find_craftable_some() {
        let mut available = HashMap::new();
        available.insert(Voxel::Wood, 10);
        available.insert(Voxel::Stone, 10);
        available.insert(Voxel::Clay, 10);

        let craftable = find_craftable(&available);
        assert_eq!(craftable.len(), 3);
        assert_eq!(craftable[0].output.voxel, Voxel::Boards);
        assert_eq!(craftable[1].output.voxel, Voxel::Cobblestone);
        assert_eq!(craftable[2].output.voxel, Voxel::Brick);
    }

    #[test]
    pub fn test_find_craftable_none() {
        let mut available = HashMap::new();
        available.insert(Voxel::Boards, 10);
        available.insert(Voxel::Glass, 220);
        available.insert(Voxel::Clay, 10);
        available.insert(Voxel::Grass, 50);
        let craftable = find_craftable(&available);
        assert_eq!(craftable.len(), 0);
    }
}
