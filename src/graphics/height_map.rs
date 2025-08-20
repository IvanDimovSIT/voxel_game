use macroquad::{
    camera::Camera3D,
    color::WHITE,
    texture::{FilterMode, Image, Texture2D},
};

use crate::{
    model::{
        area::{AREA_SIZE, AreaLocation},
        user_settings::{ShadowType, UserSettings},
        world::World,
    },
    utils::vector_to_location,
};

const MAX_LOADED_AREAS_PER_AXIS: u16 = 37;
const IMAGE_SIZE: u16 = AREA_SIZE as u16 * MAX_LOADED_AREAS_PER_AXIS;

pub struct HeightMap {
    pixel_buffer: Image,
    height_map: Texture2D,
    empty_height_map: Texture2D,
}
impl HeightMap {
    pub fn new() -> Self {
        let pixel_buffer = Image::gen_image_color(IMAGE_SIZE, IMAGE_SIZE, WHITE);
        let height_map = Texture2D::from_image(&pixel_buffer);
        Self {
            pixel_buffer,
            empty_height_map: Texture2D::from_image(&Image::gen_image_color(
                IMAGE_SIZE, IMAGE_SIZE, WHITE,
            )),
            height_map,
        }
    }

    pub fn generate_height_map(
        &mut self,
        world: &World,
        visible_area_locations: impl Iterator<Item = AreaLocation>,
        camera: &Camera3D,
        user_settings: &UserSettings,
    ) -> Texture2D {
        let area_offset: AreaLocation = vector_to_location(camera.position).into();
        let pixels = self.pixel_buffer.get_image_data_mut();

        for visible_area_location in visible_area_locations {
            let area = world.get_area_without_loading(visible_area_location);
            let area_offset_x =
                (visible_area_location.x as i32 - area_offset.x as i32) * AREA_SIZE as i32;
            let area_offset_y =
                (visible_area_location.y as i32 - area_offset.y as i32) * AREA_SIZE as i32;
            for y in 0..AREA_SIZE {
                for x in 0..AREA_SIZE {
                    let height = area.sample_height(x, y);
                    let image_x_offset = area_offset_x + x as i32;
                    let image_y_offset = area_offset_y + y as i32;
                    let image_x = IMAGE_SIZE as i32 / 2 + image_x_offset;
                    let image_y = IMAGE_SIZE as i32 / 2 + image_y_offset;
                    debug_assert!(image_x < IMAGE_SIZE as i32);
                    debug_assert!(image_y < IMAGE_SIZE as i32);
                    debug_assert!(image_x >= 0);
                    debug_assert!(image_y >= 0);
                    pixels[(image_x + image_y * IMAGE_SIZE as i32) as usize] = [height; 4];
                }
            }
        }

        self.height_map.update(&self.pixel_buffer);
        match user_settings.shadow_type {
            ShadowType::Soft => self.height_map.set_filter(FilterMode::Linear),
            ShadowType::Hard => self.height_map.set_filter(FilterMode::Nearest),
            ShadowType::None => unreachable!("Must have a dynamic shadows to generate height map"),
        }

        self.height_map.weak_clone()
    }

    pub fn get_empty_height_map(&self) -> Texture2D {
        self.empty_height_map.weak_clone()
    }
}
