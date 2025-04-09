use crate::model::{
    area::{AREA_HEIGHT, AREA_SIZE, Area, AreaLocation},
    location::{InternalLocation, Location},
    voxel::Voxel,
};

const PROBABILITY: u64 = 30;
const WOOD_LOCATIONS: [Location; 3] = [
    Location::new(0, 0, -1),
    Location::new(0, 0, -2),
    Location::new(0, 0, -3),
];
const LEAVES_LOCATIONS: [Location; 5] = [
    Location::new(0, 0, -4),
    Location::new(0, -1, -3),
    Location::new(0, 1, -3),
    Location::new(1, 0, -3),
    Location::new(-1, 0, -3),
];

fn combine_seed(seed: u64, area_location: AreaLocation, local: InternalLocation) -> u64 {
    let mut combined_seed = seed;
    combined_seed ^= area_location.x as u64;
    combined_seed ^= area_location.y as u64;
    combined_seed ^= local.x as u64;
    combined_seed ^= local.y as u64;
    combined_seed ^= local.z as u64;

    combined_seed
}

/// Xorshift algorithm
fn xorshift64(seed: u64) -> u64 {
    let mut x = seed;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

pub fn should_generate_tree(
    voxel: Voxel,
    seed: u64,
    area_location: AreaLocation,
    local: InternalLocation,
) -> bool {
    match voxel {
        Voxel::Grass => {}
        _ => return false,
    };

    let combined_seed = combine_seed(seed, area_location, local);
    let random_value = xorshift64(combined_seed);

    (random_value >> 4) % PROBABILITY == 0
}

fn can_generate_tree(area: &mut Area, local: InternalLocation) -> bool {
    WOOD_LOCATIONS
        .iter()
        .chain(LEAVES_LOCATIONS.iter())
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

fn generate_tree(area: &mut Area, local: InternalLocation) {
    for wood in WOOD_LOCATIONS.iter() {
        let offset_x = wood.x + local.x as i32;
        let offset_y = wood.y + local.y as i32;
        let offset_z = wood.z + local.z as i32;
        let offset_location =
            InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);
        area.set(offset_location, Voxel::Wood);
    }

    for leaves in LEAVES_LOCATIONS.iter() {
        let offset_x = leaves.x + local.x as i32;
        let offset_y = leaves.y + local.y as i32;
        let offset_z = leaves.z + local.z as i32;
        let offset_location =
            InternalLocation::new(offset_x as u32, offset_y as u32, offset_z as u32);
        area.set(offset_location, Voxel::Leaves);
    }
}

pub fn generate_trees(area: &mut Area, locations: &[InternalLocation]) {
    for location in locations {
        if can_generate_tree(area, *location) {
            generate_tree(area, *location);
        }
    }
}
