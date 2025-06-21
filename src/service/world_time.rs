use std::f32::consts::PI;

const LENGTH_OF_DAY: f32 = 200.0;
const LIGHT_LEVEL_COEF: f32 = -10.0;

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
        self.delta %= PI;
        self.light = Self::to_light_level(self.delta);
    }

    pub fn get_delta(&self) -> f32 {
        self.delta
    }

    pub fn get_ligth_level(&self) -> f32 {
        self.light
    }

    fn to_light_level(delta: f32) -> f32 {
        sigmoid(delta.sin(), LIGHT_LEVEL_COEF).clamp(0.1, 1.0)
    }
}

/// smoothing function
fn sigmoid(x: f32, coef: f32) -> f32 {
    1.0 / (1.0 + (coef * (x - 0.5)).exp())
}
