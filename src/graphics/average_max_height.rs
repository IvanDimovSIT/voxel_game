use crate::model::{
    area::AREA_HEIGHT, location::Location, user_settings::UserSettings, voxel::Voxel, world::World,
};

const SAMPLE_RADIUS: i32 = 2;
const SAMPLE_SIZE: i32 = (SAMPLE_RADIUS * 2 + 1) * (SAMPLE_RADIUS * 2 + 1);

pub fn calculate_average_height_around_location(
    world: &mut World,
    location: Location,
    user_settings: &UserSettings,
) -> Option<f32> {
    if !user_settings.dynamic_lighting {
        return None;
    }

    Some(
        (location.x - SAMPLE_RADIUS..=location.x + SAMPLE_RADIUS)
            .flat_map(|x| {
                (location.y - SAMPLE_RADIUS..=location.y + SAMPLE_RADIUS).map(move |y| (x, y))
            })
            .map(|(x, y)| find_max_height(world, x, y))
            .sum::<f32>()
            / SAMPLE_SIZE as f32,
    )
}

fn find_max_height(world: &mut World, x: i32, y: i32) -> f32 {
    (0..AREA_HEIGHT as i32)
        .find(|z| {
            let voxel = world.get(Location::new(x, y, *z));
            voxel != Voxel::None
        })
        .map(|max_height| max_height as f32)
        .unwrap_or(AREA_HEIGHT as f32)
}
