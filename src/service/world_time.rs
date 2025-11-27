use std::f32::consts::PI;

use crate::graphics::rain_system::RainLightLevelModifier;

const LENGTH_OF_DAY: f32 = 200.0;
const LIGHT_LEVEL_COEF: f32 = -10.0;

pub struct WorldTime {
    delta: f32,
    light: f32,
}
impl WorldTime {
    pub const MAX_LIGHT_LEVEL: f32 = 1.0;
    const MIN_LIGHT_LEVEL: f32 = 0.1;

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

    pub fn get_light_level(&self, rain_light_level_modifier: RainLightLevelModifier) -> f32 {
        match rain_light_level_modifier {
            RainLightLevelModifier::Multiply(x) => self.light * x,
            RainLightLevelModifier::Set(x) => x,
        }
        .clamp(Self::MIN_LIGHT_LEVEL, Self::MAX_LIGHT_LEVEL)
    }

    fn to_light_level(delta: f32) -> f32 {
        sigmoid(delta.sin(), LIGHT_LEVEL_COEF).clamp(Self::MIN_LIGHT_LEVEL, Self::MAX_LIGHT_LEVEL)
    }
}

/// smoothing function
fn sigmoid(x: f32, coef: f32) -> f32 {
    1.0 / (1.0 + (coef * (x - 0.5)).exp())
}

#[cfg(test)]
mod tests {
    use macroquad::rand::rand;

    use super::*;

    #[test]
    fn test_world_time() {
        let mut world_time = WorldTime::new(0.0);
        world_time.update(0.1);
        assert_in_range(&world_time);
        assert!(world_time.get_delta() > 0.0);

        world_time.update(100_000_000.0);
        assert_in_range(&world_time);

        for _ in 0..100 {
            let delta = (rand() % 10_000) as f32 / 1000.0;
            world_time.update(delta);
            assert_in_range(&world_time);
        }
    }

    fn assert_in_range(world_time: &WorldTime) {
        let delta = world_time.get_delta();
        let light = world_time.get_light_level(RainLightLevelModifier::Multiply(1.0));

        assert!(delta >= 0.0);
        assert!(delta <= PI);
        assert!(light >= 0.0);
        assert!(light <= 1.0);
    }
}
