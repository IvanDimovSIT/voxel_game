use macroquad::{color::WHITE, shapes::draw_circle};


pub fn draw_crosshair(width: f32, height: f32) {
    draw_circle(width/2.0, height/2.0, 2.0, WHITE);
}