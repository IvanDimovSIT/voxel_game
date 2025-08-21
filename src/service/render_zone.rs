use crate::model::area::AreaLocation;

const INITIAL_LOAD_RENDER_DISTANCE_REDUCTION: u32 = 6;
const INITIAL_LOAD_MINIMUM_RENDER_DISTANCE: u32 = 7;
const MAX_RENDER_SIZE: u32 = 100;
const LOAD_EXTRA: u32 = 2;

/// returns a list of areas to generate meshes for
pub fn get_render_zone(area_location: AreaLocation, render_size: u32) -> Vec<AreaLocation> {
    debug_assert!(
        render_size <= MAX_RENDER_SIZE,
        "render_size too large {render_size}"
    );
    if render_size == 0 {
        return vec![area_location];
    }
    let render_size = render_size as i32;

    let mut areas = Vec::with_capacity((render_size * render_size) as usize);
    for x in (-render_size)..=(render_size) {
        for y in (-render_size)..=(render_size) {
            let location_to_render = AreaLocation::new(
                (area_location.x as i32 + x) as u32,
                (area_location.y as i32 + y) as u32,
            );
            areas.push(location_to_render);
        }
    }

    areas
}

/// returns a list of areas to generate meshes for uppon initial loading of the world
pub fn get_render_zone_on_world_load(
    area_location: AreaLocation,
    render_size: u32,
) -> Vec<AreaLocation> {
    let reduced_render_size = render_size
        .saturating_sub(INITIAL_LOAD_RENDER_DISTANCE_REDUCTION)
        .max(INITIAL_LOAD_MINIMUM_RENDER_DISTANCE);
    get_render_zone(area_location, reduced_render_size)
}

/// returns a list of areas to be loaded from disk
pub fn get_load_zone(area_location: AreaLocation, render_size: u32) -> Vec<AreaLocation> {
    get_render_zone(area_location, render_size + LOAD_EXTRA)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_get_render_zone() {
        let render_zone = get_render_zone(AreaLocation::new(10, 10), 2);
        assert_eq!(render_zone.len(), 25);
        for location in render_zone {
            assert!((location.x as i32 - 10).abs() <= 2);
            assert!((location.y as i32 - 10).abs() <= 2);
        }
    }

    #[test]
    pub fn test_get_render_zone_on_world_load_less_than_minimum() {
        let render_zone = get_render_zone_on_world_load(AreaLocation::new(10, 10), 9);
        const MINIMUM_AREAS_SIDE: u32 = INITIAL_LOAD_MINIMUM_RENDER_DISTANCE*2+1;
        const MINIMUM_AREAS: u32 = MINIMUM_AREAS_SIDE*MINIMUM_AREAS_SIDE;
        assert_eq!(render_zone.len() as u32, MINIMUM_AREAS);
        for location in render_zone {
            assert!((location.x as i32 - 10).abs() as u32 <= INITIAL_LOAD_MINIMUM_RENDER_DISTANCE);
            assert!((location.y as i32 - 10).abs() as u32 <= INITIAL_LOAD_MINIMUM_RENDER_DISTANCE);
        }
    }

    #[test]
    pub fn test_get_render_zone_on_world_load_greater_than_minimum() {
        let render_zone = get_render_zone_on_world_load(AreaLocation::new(10, 10), 16);
        const AREAS_SIDE: u32 = 1 + 2*(16 - INITIAL_LOAD_RENDER_DISTANCE_REDUCTION);
        const AREAS_SIZE: u32 = AREAS_SIDE*AREAS_SIDE;
        assert_eq!(render_zone.len() as u32, AREAS_SIZE);
        for location in render_zone {
            assert!((location.x as i32 - 10).abs() as u32 <= AREAS_SIDE);
            assert!((location.y as i32 - 10).abs() as u32 <= AREAS_SIDE);
        }
    }


    #[test]
    pub fn test_get_load_zone() {
        let load_zone = get_load_zone(AreaLocation::new(10, 10), 2);
        let side_of_zone = 1 + 2 * (2 + LOAD_EXTRA);
        let area_of_zone = side_of_zone * side_of_zone;
        assert_eq!(load_zone.len(), area_of_zone as usize);
        for location in load_zone {
            assert!((location.x as i32 - 10).abs() <= 2 + LOAD_EXTRA as i32);
            assert!((location.y as i32 - 10).abs() <= 2 + LOAD_EXTRA as i32);
        }
    }
}
