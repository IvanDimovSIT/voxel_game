use bincode::{Decode, Encode};

#[derive(Debug, Clone, Copy, Encode, Decode)]
pub struct ActivityTimer {
    delta: f32,
    cooldown: f32
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
}
