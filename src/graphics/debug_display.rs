use macroquad::{
    camera::{Camera3D, set_default_camera},
    color::WHITE,
    prelude::info,
    text::draw_text,
    time::{get_fps, get_frame_time},
};

use crate::{
    model::{area::VOXELS_IN_AREA, voxel::Voxel, world::World},
    service::camera_controller::CameraController, utils::vector_to_location,
};

use super::renderer::Renderer;

const KILOBYTE: usize = 1024;
const FONT_SIZE: f32 = 30.0;

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
        rendered_areas_faces: (usize, usize)
    ) {
        if !self.should_display {
            return;
        }

        let fps = get_fps();
        let frame_time_ms = get_frame_time() * 1000.0;
        let meshes = renderer.get_voxel_face_count();
        let camera_location = vector_to_location(camera.position);
        let look_target = camera.target;
        let loaded_areas = world.get_loaded_areas_count();
        let areas_memory_kb = loaded_areas * size_of::<Voxel>() * VOXELS_IN_AREA / KILOBYTE;

        set_default_camera();
        draw_text(
            &format!("FPS:{fps}({frame_time_ms:.2}ms)"),
            10.0,
            FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!("Voxel faces:{meshes}"),
            10.0,
            2.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!("Visible: {} ({} areas)", rendered_areas_faces.1, rendered_areas_faces.0),
            10.0,
            3.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!("Loaded areas:{loaded_areas}({areas_memory_kb}KB)"),
            10.0,
            4.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!(
                "X:{:.2}, Y:{:.2}, Z:{:.2}",
                camera.position.x, camera.position.y, camera.position.z
            ),
            10.0,
            5.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!(
                "(X:{}, Y:{}, Z:{})",
                camera_location.x, camera_location.y, camera_location.z
            ),
            10.0,
            6.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
        draw_text(
            &format!(
                "Look: X:{:.2}, Y:{:.2}, Z:{:.2}",
                look_target.x, look_target.y, look_target.z
            ),
            10.0,
            7.0 * FONT_SIZE,
            FONT_SIZE,
            WHITE,
        );
    }
}
