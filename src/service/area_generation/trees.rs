use crate::{
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, Area},
        location::{AreaLocation, InternalLocation, Location},
        voxel::Voxel,
    },
    service::area_generation::algorithms::sample_probability,
    utils::StackVec,
};

use super::algorithms::{combine_seed, split_mix64};

const ALLOWED_TREE_BASES: [Voxel; 4] = [Voxel::Grass, Voxel::Dirt, Voxel::Clay, Voxel::Sand];
const BASE_PROBABILITY: u64 = 60;
const SHORT_TREE_LOCATIONS: [(Location, Voxel); 8] = [
    (Location::new(0, 0, -1), Voxel::Wood),
    (Location::new(0, 0, -2), Voxel::Wood),
    (Location::new(0, 0, -3), Voxel::Wood),
    (Location::new(0, 0, -4), Voxel::Leaves),
    (Location::new(0, -1, -3), Voxel::Leaves),
    (Location::new(0, 1, -3), Voxel::Leaves),
    (Location::new(1, 0, -3), Voxel::Leaves),
    (Location::new(-1, 0, -3), Voxel::Leaves),
];
const TALL_TREE_LOCATIONS: [(Location, Voxel); 9] = [
    (Location::new(0, 0, -1), Voxel::Wood),
    (Location::new(0, 0, -2), Voxel::Wood),
    (Location::new(0, 0, -3), Voxel::Wood),
    (Location::new(0, 0, -4), Voxel::Wood),
    (Location::new(0, 0, -5), Voxel::Leaves),
    (Location::new(0, -1, -4), Voxel::Leaves),
    (Location::new(0, 1, -4), Voxel::Leaves),
    (Location::new(1, 0, -4), Voxel::Leaves),
    (Location::new(-1, 0, -4), Voxel::Leaves),
];
const HUGE_TREE_LOCATIONS: [(Location, Voxel); 26] = [
    (Location::new(0, 0, -1), Voxel::Wood),
    (Location::new(0, 0, -2), Voxel::Wood),
    (Location::new(0, 0, -3), Voxel::Wood),
    (Location::new(0, 0, -4), Voxel::Wood),
    (Location::new(1, 0, -4), Voxel::Wood),
    (Location::new(0, 1, -4), Voxel::Wood),
    (Location::new(0, -1, -4), Voxel::Wood),
    (Location::new(-1, 0, -4), Voxel::Wood),
    (Location::new(0, -1, -5), Voxel::Leaves),
    (Location::new(0, -2, -5), Voxel::Leaves),
    (Location::new(0, 1, -5), Voxel::Leaves),
    (Location::new(0, 2, -5), Voxel::Leaves),
    (Location::new(1, 0, -5), Voxel::Leaves),
    (Location::new(2, 0, -5), Voxel::Leaves),
    (Location::new(-1, 0, -5), Voxel::Leaves),
    (Location::new(-2, 0, -5), Voxel::Leaves),
    (Location::new(-1, -1, -5), Voxel::Leaves),
    (Location::new(-1, 1, -5), Voxel::Leaves),
    (Location::new(1, 1, -5), Voxel::Leaves),
    (Location::new(1, -1, -5), Voxel::Leaves),
    (Location::new(0, 0, -6), Voxel::Leaves),
    (Location::new(0, 1, -6), Voxel::Leaves),
    (Location::new(1, 0, -6), Voxel::Leaves),
    (Location::new(0, -1, -6), Voxel::Leaves),
    (Location::new(-1, 0, -6), Voxel::Leaves),
    (Location::new(0, 0, -7), Voxel::Leaves),
];
const TALL_CACTUS_LOCATIONS: [(Location, Voxel); 3] = [
    (Location::new(0, 0, -1), Voxel::Cactus),
    (Location::new(0, 0, -2), Voxel::Cactus),
    (Location::new(0, 0, -3), Voxel::Cactus),
];
const DEAD_TREE_LOCATIONS: [(Location, Voxel); 2] = [
    (Location::new(0, 0, -1), Voxel::Wood),
    (Location::new(0, 0, -2), Voxel::Wood),
];
const SHORT_CACTUS_LOCATIONS: [(Location, Voxel); 1] = [(Location::new(0, 0, -1), Voxel::Cactus)];
const BUSH_LOCATIONS: [(Location, Voxel); 1] = [(Location::new(0, 0, -1), Voxel::Leaves)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeType {
    None,
    Short,
    Tall,
    TallCactus,
    DeadTree,
    HugeTree,
    ShortCactus,
    Bush,
}
impl TreeType {
    const ALL_TYPES_COUNT: usize = 7;
    const ALL_TYPES: [TreeType; Self::ALL_TYPES_COUNT] = [
        Self::Short,
        Self::Tall,
        Self::TallCactus,
        Self::ShortCactus,
        Self::DeadTree,
        Self::HugeTree,
        Self::Bush,
    ];

    fn get_voxels_for_tree_type(self) -> &'static [(Location, Voxel)] {
        debug_assert!(self != TreeType::None);

        match self {
            TreeType::None => unreachable!(),
            TreeType::Short => SHORT_TREE_LOCATIONS.as_slice(),
            TreeType::Tall => TALL_TREE_LOCATIONS.as_slice(),
            TreeType::TallCactus => TALL_CACTUS_LOCATIONS.as_slice(),
            TreeType::ShortCactus => SHORT_CACTUS_LOCATIONS.as_slice(),
            TreeType::DeadTree => DEAD_TREE_LOCATIONS.as_slice(),
            TreeType::HugeTree => HUGE_TREE_LOCATIONS.as_slice(),
            TreeType::Bush => BUSH_LOCATIONS.as_slice(),
        }
    }

    fn get_allowed_base(self) -> &'static [Voxel] {
        match self {
            TreeType::None => unreachable!(),
            TreeType::Short | TreeType::Tall => &[Voxel::Grass, Voxel::Clay],
            TreeType::TallCactus | TreeType::ShortCactus => &[Voxel::Sand],
            TreeType::DeadTree => &[Voxel::Grass, Voxel::Clay, Voxel::Dirt, Voxel::Sand],
            TreeType::HugeTree => &[Voxel::Grass],
            TreeType::Bush => &[Voxel::Grass, Voxel::Sand],
        }
    }

    fn weight(self) -> u64 {
        debug_assert!(self != TreeType::None);

        match self {
            TreeType::None => unreachable!(),
            TreeType::Short => 1000,
            TreeType::Tall => 1000,
            TreeType::TallCactus => 500,
            TreeType::ShortCactus => 200,
            TreeType::Bush => 100,
            TreeType::DeadTree => 10,
            TreeType::HugeTree => 300,
        }
    }
}

pub fn should_generate_tree(
    voxel: Voxel,
    seed: u64,
    area_location: AreaLocation,
    local: InternalLocation,
) -> TreeType {
    if !ALLOWED_TREE_BASES.contains(&voxel) {
        return TreeType::None;
    }

    let combined_seed = combine_seed(seed, area_location, local);
    let random_value = split_mix64(combined_seed);

    if !sample_probability(random_value >> 4, BASE_PROBABILITY) {
        return TreeType::None;
    }

    let mut possible_trees = StackVec::new();
    get_possible_trees(voxel, &mut possible_trees);
    if possible_trees.is_empty() {
        return TreeType::None;
    }

    let select_tree_random_value = split_mix64(random_value) >> 4;

    select_weighted_random_tree(&possible_trees, select_tree_random_value)
}

fn select_weighted_random_tree(
    possible_trees: &StackVec<TreeType, { TreeType::ALL_TYPES_COUNT }>,
    random_value: u64,
) -> TreeType {
    let total_weight: u64 = possible_trees.iter().map(|t| t.weight()).sum();

    let mut selected_weight = random_value % total_weight;

    for tree in possible_trees.iter() {
        let tree_weight = tree.weight();
        if tree_weight > selected_weight {
            return *tree;
        } else {
            selected_weight -= tree_weight;
        }
    }

    TreeType::None
}

fn get_possible_trees(voxel: Voxel, vec: &mut StackVec<TreeType, { TreeType::ALL_TYPES_COUNT }>) {
    for tree_type in TreeType::ALL_TYPES {
        if tree_type.get_allowed_base().contains(&voxel) {
            vec.push(tree_type);
        }
    }
}

pub fn generate_trees(area: &mut Area, locations: &[(InternalLocation, TreeType)]) {
    for (location, tree_type) in locations {
        if can_generate_tree(area, *location, *tree_type) {
            generate_tree(area, *location, *tree_type);
        }
    }
}

fn can_generate_tree(area: &mut Area, local: InternalLocation, tree_type: TreeType) -> bool {
    debug_assert!(tree_type != TreeType::None);
    let voxel_locations = tree_type.get_voxels_for_tree_type();

    voxel_locations.iter().all(|(location, _voxel)| {
        let offset_x = location.x + local.x as i32;
        let offset_y = location.y + local.y as i32;
        let offset_z = location.z + local.z as i32;
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
    let voxels = tree_type.get_voxels_for_tree_type();

    for (voxel_loc, voxel) in voxels.iter() {
        let offset_x = voxel_loc.x + local.x as i32;
        let offset_y = voxel_loc.y + local.y as i32;
        let offset_z = voxel_loc.z + local.z as i32;
        let offset_location =
            InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);
        area.set_without_updating_max_height(offset_location, *voxel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_generate_tree_true() {
        let mut area = Area::new(AreaLocation::new(0, 0));

        assert!(can_generate_tree(
            &mut area,
            InternalLocation::new(5, 5, 10),
            TreeType::Tall
        ));
    }

    #[test]
    fn test_can_generate_tree_false() {
        let mut area = Area::new(AreaLocation::new(0, 0));
        area.set(InternalLocation::new(5, 5, 8), Voxel::Brick);

        assert!(!can_generate_tree(
            &mut area,
            InternalLocation::new(5, 5, 10),
            TreeType::Tall
        ));
        assert!(!can_generate_tree(
            &mut area,
            InternalLocation::new(5, 5, 1),
            TreeType::Tall
        ));
    }

    #[test]
    fn test_generate_tree() {
        for tree in TreeType::ALL_TYPES {
            let mut area = Area::new(AreaLocation::new(0, 0));
            let generate_location =
                InternalLocation::new(AREA_SIZE / 2, AREA_SIZE / 2, AREA_HEIGHT / 2);
            generate_tree(&mut area, generate_location, tree);

            let x = generate_location.x as i32;
            let y = generate_location.y as i32;
            let z = generate_location.z as i32;

            let voxels_to_check = tree.get_voxels_for_tree_type();
            for (location, voxel) in voxels_to_check {
                assert_ne!(*voxel, Voxel::None);

                let voxel_location = InternalLocation::new(
                    (location.x + x) as u32,
                    (location.y + y) as u32,
                    (location.z + z) as u32,
                );
                assert_eq!(area.get(voxel_location), *voxel);
            }
        }
    }
}
