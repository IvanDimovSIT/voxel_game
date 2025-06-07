use macroquad::{
    color::{BEIGE, Color, DARKBLUE},
    window::clear_background,
};

use crate::service::world_time::WorldTime;

const BRIGHT_COLOR: Color = BEIGE;
const DARK_COLOR: Color = DARKBLUE;

pub fn draw_sky(world_time: &WorldTime) {
    let light_level = world_time.get_ligth_level();
    let dark_level = 1.0 - light_level;
    let sky_color = Color::new(
        BRIGHT_COLOR.r * light_level + DARK_COLOR.r * dark_level,
        BRIGHT_COLOR.g * light_level + DARK_COLOR.g * dark_level,
        BRIGHT_COLOR.b * light_level + DARK_COLOR.b * dark_level,
        1.0,
    );
    clear_background(sky_color);
}
