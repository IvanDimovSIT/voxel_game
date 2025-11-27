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

    pub fn reset(&mut self) {
        self.delta = 0.0;
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

    /// returns true if the activity is triggered, changes cooldown based on closure result if triggered
    pub fn tick_change_cooldown(&mut self, delta: f32, new_cooldown_fn: impl Fn() -> f32) -> bool {
        self.delta += delta;

        if self.delta >= self.cooldown {
            self.delta -= self.cooldown;
            self.cooldown = new_cooldown_fn();

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

    #[test]
    fn test_tick_change_cooldown_not_triggered() {
        let mut timer = ActivityTimer::new(0.0, 1.0);

        let triggered = timer.tick_change_cooldown(0.5, || 2.0);
        assert!(!triggered);
        assert_eq!(timer.get_delta(), 0.5);
        assert_eq!(timer.cooldown, 1.0);
    }

    #[test]
    fn test_tick_change_cooldown_triggered_once() {
        let mut timer = ActivityTimer::new(0.0, 1.0);

        let triggered = timer.tick_change_cooldown(1.2, || 2.0);
        assert!(triggered);
        assert!((timer.get_delta() - 0.2).abs() <= 0.001);
        assert_eq!(timer.cooldown, 2.0);
    }

    #[test]
    fn test_tick_change_cooldown_multiple_triggers() {
        let mut timer = ActivityTimer::new(0.0, 1.0);

        let triggered = timer.tick_change_cooldown(1.0, || 2.0);
        assert!(triggered);
        assert_eq!(timer.get_delta(), 0.0);
        assert_eq!(timer.cooldown, 2.0);

        let triggered = timer.tick_change_cooldown(1.5, || 1.5);
        assert!(!triggered);
        assert_eq!(timer.get_delta(), 1.5);
        assert_eq!(timer.cooldown, 2.0);

        let triggered = timer.tick_change_cooldown(1.0, || 0.5);
        assert!(triggered);
        assert_eq!(timer.get_delta(), 0.5);
        assert_eq!(timer.cooldown, 0.5);
    }
}
