use std::time::Instant;

use macroquad::{camera::Camera3D, color::{Color, WHITE}, texture::{Image, Texture2D}};

use crate::{model::{area::{Area, AreaLocation, AREA_SIZE}, world::World}, utils::vector_to_location};

const MAX_LOADED_AREAS_PER_AXIS: u16 = 37;
const IMAGE_SIZE: u16 = AREA_SIZE as u16 * MAX_LOADED_AREAS_PER_AXIS;

pub fn generate_height_map(world: &World, visible_areas: Vec<AreaLocation>, camera: &Camera3D) -> Texture2D {
    let start = Instant::now();
    let mut image = Image::gen_image_color(IMAGE_SIZE, IMAGE_SIZE, WHITE);
    let area_offset: AreaLocation = vector_to_location(camera.position).into();
    for visible_area in visible_areas {
        let max_height = world.get_area(visible_area).get_max_height();
        for y in 0..AREA_SIZE {
            for x in 0..AREA_SIZE {
                let height = Area::sample_height(max_height, x, y);
                let image_x_offset = (visible_area.x as i32 - area_offset.x as i32) * AREA_SIZE as i32 + x as i32; 
                let image_y_offset = (visible_area.y as i32 - area_offset.y as i32) * AREA_SIZE as i32 + y as i32;
                let image_x = IMAGE_SIZE as i32/2 + image_x_offset; 
                let image_y = IMAGE_SIZE as i32/2 + image_y_offset; 
                debug_assert!(image_x < IMAGE_SIZE as i32);
                debug_assert!(image_y < IMAGE_SIZE as i32);
                debug_assert!(image_x >= 0);
                debug_assert!(image_y >= 0);
                image.set_pixel(image_x as u32, image_y as u32, Color::from_rgba(height, height, height, height));
            }   
        }
    }
    let end = start.elapsed();
    println!("Height map: {}us", end.as_micros());
    
    Texture2D::from_image(&image)
}

pub fn generate_empty_height_map() -> Texture2D {
    Texture2D::from_image(&Image::gen_image_color(IMAGE_SIZE, IMAGE_SIZE, WHITE))
}