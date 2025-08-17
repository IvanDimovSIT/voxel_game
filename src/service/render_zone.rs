use crate::model::area::AreaLocation;

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
