use crate::model::area::AreaLocation;

const MAX_RENDER_SIZE: u32 = 100;
const LOAD_EXTRA: u32 = 2;

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

pub fn get_load_zone(area_location: AreaLocation, render_size: u32) -> Vec<AreaLocation> {
    get_render_zone(area_location, render_size + LOAD_EXTRA)
}
