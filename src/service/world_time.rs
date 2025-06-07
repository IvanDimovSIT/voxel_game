use std::f32::consts::PI;

const LENGTH_OF_DAY: f32 = 100.0;

pub struct WorldTime {
    delta: f32,
    light: f32,
}
impl WorldTime {
    pub fn new(delta: f32) -> Self {
        Self {
            delta,
            light: Self::to_light_level(delta),
        }
    }

    pub fn update(&mut self, delta: f32) {
        self.delta += delta / LENGTH_OF_DAY;
        self.delta = self.delta % PI;
        self.light = Self::to_light_level(self.delta);
    }

    pub fn get_ligth_level(&self) -> f32 {
        self.light
    }

    fn to_light_level(delta: f32) -> f32 {
        (delta.sin().sqrt() * 1.2).clamp(0.1, 1.0)
    }
}
