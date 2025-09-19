use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct ActivityTimer {
    delta: f32,
    cooldown: f32,
}
impl ActivityTimer {
    pub fn new(delta: f32, cooldown: f32) -> Self {
        debug_assert!(cooldown > 0.0);
        Self { delta, cooldown }
    }

    /// returns true if the activity is triggered
    pub fn tick(&mut self, delta: f32) -> bool {
        self.delta += delta;

        if self.delta >= self.cooldown {
            self.delta -= self.cooldown;

            true
        } else {
            false
        }
    }

    pub fn get_delta(&self) -> f32 {
        self.delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_timer() {
        let mut timer = ActivityTimer::new(0.0, 1.0);

        assert_eq!(timer.get_delta(), 0.0);
        assert!(!timer.tick(0.75));
        assert_eq!(timer.get_delta(), 0.75);
        assert!(timer.tick(0.75));
        assert_eq!(timer.get_delta(), 0.5);
    }
}
