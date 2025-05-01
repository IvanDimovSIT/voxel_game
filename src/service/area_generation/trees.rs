use crate::model::{
    area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
    location::{InternalLocation, Location},
    voxel::Voxel,
};

use super::algorithms::{combine_seed, split_mix64};

const PROBABILITY: u64 = 60;
const SHORT_WOOD_LOCATIONS: [Location; 3] = [
    Location::new(0, 0, -1),
    Location::new(0, 0, -2),
    Location::new(0, 0, -3),
];
const SHORT_LEAVES_LOCATIONS: [Location; 5] = [
    Location::new(0, 0, -4),
    Location::new(0, -1, -3),
    Location::new(0, 1, -3),
    Location::new(1, 0, -3),
    Location::new(-1, 0, -3),
];
const TALL_WOOD_LOCATIONS: [Location; 4] = [
    Location::new(0, 0, -1),
    Location::new(0, 0, -2),
    Location::new(0, 0, -3),
    Location::new(0, 0, -4),
];
const TALL_LEAVES_LOCATIONS: [Location; 5] = [
    Location::new(0, 0, -5),
    Location::new(0, -1, -4),
    Location::new(0, 1, -4),
    Location::new(1, 0, -4),
    Location::new(-1, 0, -4),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    None,
    Short,
    Tall,
}

pub fn should_generate_tree(
    voxel: Voxel,
    seed: u64,
    area_location: AreaLocation,
    local: InternalLocation,
) -> TreeType {
    match voxel {
        Voxel::Grass => {}
        _ => return TreeType::None,
    };

    let combined_seed = combine_seed(seed, area_location, local);
    let mut random_value = split_mix64(combined_seed);

    if (random_value >> 4) % PROBABILITY == 0 {
        random_value = split_mix64(random_value);
        if (random_value >> 4) % 2 == 0 {
            TreeType::Short
        } else {
            TreeType::Tall
        }
    } else {
        TreeType::None
    }
}

fn get_leaves_locations_for_tree_type(tree_type: TreeType) -> &'static [Location] {
    debug_assert!(tree_type != TreeType::None);

    match tree_type {
        TreeType::None => unreachable!(),
        TreeType::Short => SHORT_LEAVES_LOCATIONS.as_slice(),
        TreeType::Tall => TALL_LEAVES_LOCATIONS.as_slice(),
    }
}

fn get_wood_locations_for_tree_type(tree_type: TreeType) -> &'static [Location] {
    debug_assert!(tree_type != TreeType::None);

    match tree_type {
        TreeType::None => unreachable!(),
        TreeType::Short => SHORT_WOOD_LOCATIONS.as_slice(),
        TreeType::Tall => TALL_WOOD_LOCATIONS.as_slice(),
    }
}

fn can_generate_tree(area: &mut Area, local: InternalLocation, tree_type: TreeType) -> bool {
    debug_assert!(tree_type != TreeType::None);
    let wood_locations = get_wood_locations_for_tree_type(tree_type);
    let leaves_location = get_leaves_locations_for_tree_type(tree_type);

    wood_locations
        .iter()
        .chain(leaves_location.iter())
        .all(|voxel| {
            let offset_x = voxel.x + local.x as i32;
            let offset_y = voxel.y + local.y as i32;
            let offset_z = voxel.z + local.z as i32;
            if offset_x < 0 || offset_x >= AREA_SIZE as i32 {
                return false;
            }
            if offset_y < 0 || offset_y >= AREA_SIZE as i32 {
                return false;
            }
            if offset_z < 0 || offset_z >= AREA_HEIGHT as i32 {
                return false;
            }
            let offset_location =
                InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);

            area.get(offset_location) == Voxel::None
        })
}

fn generate_tree(area: &mut Area, local: InternalLocation, tree_type: TreeType) {
    let wood_locations = get_wood_locations_for_tree_type(tree_type);
    let leaves_location = get_leaves_locations_for_tree_type(tree_type);

    for wood in wood_locations.iter() {
        let offset_x = wood.x + local.x as i32;
        let offset_y = wood.y + local.y as i32;
        let offset_z = wood.z + local.z as i32;
        let offset_location =
            InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);
        area.set(offset_location, Voxel::Wood);
    }

    for leaves in leaves_location.iter() {
        let offset_x = leaves.x + local.x as i32;
        let offset_y = leaves.y + local.y as i32;
        let offset_z = leaves.z + local.z as i32;
        let offset_location =
            InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);
        area.set(offset_location, Voxel::Leaves);
    }
}

pub fn generate_trees(area: &mut Area, locations: &[(InternalLocation, TreeType)]) {
    for (location, tree_type) in locations {
        if can_generate_tree(area, *location, *tree_type) {
            generate_tree(area, *location, *tree_type);
        }
    }
}
