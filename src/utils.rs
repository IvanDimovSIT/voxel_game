use std::sync::{Condvar, Mutex};

use macroquad::math::Vec3;

use crate::model::{area::AREA_HEIGHT, location::Location};

pub struct Semaphore {
    count: Mutex<usize>,
    condvar: Condvar,
}
impl Semaphore {
    pub const fn new(count: usize) -> Self {
        Self {
            count: Mutex::new(count),
            condvar: Condvar::new(),
        }
    }

    pub fn acquire(&self) {
        let mut count = self.count.lock().unwrap();
        while *count == 0 {
            count = self.condvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    pub fn release(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        self.condvar.notify_one();
    }
}

pub fn vector_to_location(vec3: Vec3) -> Location {
    let x = vec3.x.round() as i32;
    let y = vec3.y.round() as i32;
    let z = (vec3.z.round() as i32).clamp(0, AREA_HEIGHT as i32 - 1);

    Location::new(x, y, z)
}
