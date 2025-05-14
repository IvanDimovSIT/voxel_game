const MIN_RENDER_DISTANCE: u32 = 3;
const MAX_RENDER_DISTANCE: u32 = 14;

#[derive(Debug, Clone)]
pub struct UserSettings {
    render_distance: u32,
    pub has_sound: bool,
    pub is_fullscreen: bool
}
impl UserSettings {
    pub fn get_render_distance(&self) -> u32 {
        self.render_distance
    }

    pub fn increase_render_distance(&mut self) -> bool {
        if self.render_distance < MAX_RENDER_DISTANCE {
            self.render_distance += 1;
            true
        } else {
            false
        }
    }

    pub fn decrease_render_distance(&mut self) -> bool {
        if self.render_distance > MIN_RENDER_DISTANCE {
            self.render_distance -= 1;
            true
        } else {
            false
        }
    }
}
impl Default for UserSettings {
    fn default() -> Self {
        Self {
            render_distance: 7,
            has_sound: true,
            is_fullscreen: false,
        }
    }
}
