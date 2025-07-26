use crate::model::{location::Location, user_settings::UserSettings, world::World};

const SAMPLE_RADIUS: i32 = 2;

pub fn calculate_max_height_around_location(
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
            .fold(0.0, |acc, x| if x > acc { x } else { acc }),
    )
}

fn find_max_height(world: &mut World, x: i32, y: i32) -> f32 {
    world.find_max_height_of_opaque_voxels_for_column(Location::new(x, y, 0)) as f32
}
