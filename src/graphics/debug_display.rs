use macroquad::{
    camera::Camera3D,
    color::WHITE,
    math::vec3,
    models::draw_cube_wires,
    prelude::info,
    shapes::draw_rectangle,
    text::Font,
    time::{get_fps, get_frame_time},
};

use crate::{
    interface::{
        style::{CLEAR_SCREEN_COLOR, TEXT_COLOR},
        util::draw_game_text,
    },
    model::{
        area::{AREA_HEIGHT, AREA_SIZE, VOXELS_IN_AREA},
        voxel::Voxel,
        world::World,
    },
    service::camera_controller::CameraController,
    utils::vector_to_location,
};

use super::renderer::Renderer;

const KILOBYTE: usize = 1024;
const MS_IN_SECONDS: f32 = 1000.0;
const FONT_SIZE: f32 = 30.0;
const LEFT_MARGIN: f32 = 10.0;

pub struct DebugDisplay {
    should_display: bool,
}
impl DebugDisplay {
    pub fn new() -> Self {
        Self {
            should_display: false,
        }
    }

    pub fn toggle_display(&mut self) {
        self.should_display = !self.should_display;
        info!("Debug display:{}", self.should_display);
    }

    pub fn draw_debug_display(
        &self,
        world: &World,
        renderer: &Renderer,
        camera: &Camera3D,
        rendered_areas_faces: (usize, usize),
        font: &Font,
    ) {
        if !self.should_display {
            return;
        }

        let fps = get_fps();
        let frame_time_ms = get_frame_time() * MS_IN_SECONDS;
        let meshes = renderer.get_voxel_face_count();
        let camera_location = vector_to_location(camera.position);
        let look_target = camera.target;
        let loaded_areas = world.get_loaded_areas_count();
        let areas_max_height_bytes = loaded_areas * VOXELS_IN_AREA;
        let areas_voxels_bytes = loaded_areas * size_of::<Voxel>() * VOXELS_IN_AREA;
        let areas_memory_kb = (areas_max_height_bytes + areas_voxels_bytes) / KILOBYTE;
        let waiting_to_be_rendered = renderer.get_areas_waiting_to_be_rendered();

        Self::draw_background();
        draw_game_text(
            &format!("FPS:{fps}({frame_time_ms:.2}ms)"),
            LEFT_MARGIN,
            FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!("Voxel faces:{meshes}"),
            LEFT_MARGIN,
            2.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!(
                "Visible: {} Areas:{} ({waiting_to_be_rendered} waiting)",
                rendered_areas_faces.1, rendered_areas_faces.0
            ),
            LEFT_MARGIN,
            3.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!("Loaded areas:{loaded_areas}({areas_memory_kb}KB)"),
            LEFT_MARGIN,
            4.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!(
                "X:{:.2}, Y:{:.2}, Z:{:.2}",
                camera.position.x, camera.position.y, camera.position.z
            ),
            LEFT_MARGIN,
            5.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!(
                "(X:{}, Y:{}, Z:{})",
                camera_location.x, camera_location.y, camera_location.z
            ),
            LEFT_MARGIN,
            6.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
        draw_game_text(
            &format!(
                "Look: X:{:.2}, Y:{:.2}, Z:{:.2}",
                look_target.x, look_target.y, look_target.z
            ),
            LEFT_MARGIN,
            7.0 * FONT_SIZE,
            FONT_SIZE,
            TEXT_COLOR,
            font,
        );
    }

    pub fn draw_area_border(&self, camera_controller: &CameraController) {
        if !self.should_display {
            return;
        }

        const AREA_SIZE_F32: f32 = AREA_SIZE as f32;
        const AREA_SIZE_I32: i32 = AREA_SIZE as i32;

        let camera_location = camera_controller.get_camera_voxel_location();
        let area_x = camera_location.x.div_euclid(AREA_SIZE as i32);
        let area_y = camera_location.y.div_euclid(AREA_SIZE as i32);

        let x = (area_x * AREA_SIZE_I32) as f32 + AREA_SIZE_F32 / 2.0 - 0.5;
        let y = (area_y * AREA_SIZE_I32) as f32 + AREA_SIZE_F32 / 2.0 - 0.5;
        let z = AREA_HEIGHT as f32 / 2.0;
        let size = AREA_SIZE_F32;
        let height = AREA_HEIGHT as f32;

        let position = vec3(x, y, z - 0.5) - camera_controller.get_position();
        draw_cube_wires(position, vec3(size, size, height), WHITE);
    }

    fn draw_background() {
        draw_rectangle(0.0, 0.0, 530.0, FONT_SIZE * 8.0, CLEAR_SCREEN_COLOR);
    }
}
