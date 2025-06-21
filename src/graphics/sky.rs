use macroquad::{color::Color, window::clear_background};

use crate::service::world_time::WorldTime;

pub const SKY_BRIGHT_COLOR: Color = Color::new(0.92, 0.94, 0.61, 1.0);
pub const SKY_DARK_COLOR: Color = Color::new(0.12, 0.08, 0.36, 1.0);

pub fn draw_sky(world_time: &WorldTime) {
    let light_level = world_time.get_ligth_level();
    let dark_level = 1.0 - light_level;
    let sky_color = Color::new(
        SKY_BRIGHT_COLOR.r * light_level + SKY_DARK_COLOR.r * dark_level,
        SKY_BRIGHT_COLOR.g * light_level + SKY_DARK_COLOR.g * dark_level,
        SKY_BRIGHT_COLOR.b * light_level + SKY_DARK_COLOR.b * dark_level,
        1.0,
    );
    clear_background(sky_color);
}
