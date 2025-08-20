use std::{
    cell::RefCell,
    mem::MaybeUninit,
    ops::Deref,
    sync::{Condvar, Mutex},
};

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

pub struct StackVec<T, const MAX: usize>
where
    T: Copy,
{
    size: usize,
    array: [MaybeUninit<T>; MAX],
}
impl<T, const MAX: usize> StackVec<T, MAX>
where
    T: Copy,
{
    pub fn new() -> Self {
        Self {
            size: 0,
            array: [MaybeUninit::uninit(); MAX],
        }
    }

    pub fn push(&mut self, value: T) {
        debug_assert!(self.size < MAX, "Array is at max size");
        self.array[self.size].write(value);
        self.size += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
impl<T, const MAX: usize> Deref for StackVec<T, MAX>
where
    T: Copy,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.array.as_ptr() as *const T, self.size) }
    }
}
impl<T, const MAX: usize> IntoIterator for StackVec<T, MAX>
where
    T: Copy,
{
    type Item = T;

    type IntoIter = StackVecIterator<T, MAX>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            stack_vec: self,
            index: 0,
        }
    }
}
pub struct StackVecIterator<T, const MAX: usize>
where
    T: Copy,
{
    stack_vec: StackVec<T, MAX>,
    index: usize,
}
impl<T, const MAX: usize> Iterator for StackVecIterator<T, MAX>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.stack_vec.size {
            None
        } else {
            let result = self.stack_vec[self.index];
            self.index += 1;
            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.stack_vec.array.len() - self.index,
            Some(self.stack_vec.array.len() - self.index),
        )
    }
}
impl<T, const MAX: usize> ExactSizeIterator for StackVecIterator<T, MAX> where T: Copy {}

thread_local! {
    static STRING_BUFFER: RefCell<String> = RefCell::new(String::with_capacity(64));
}
pub fn use_str_buffer(f: impl Fn(&mut String)) {
    STRING_BUFFER.with(|buffer| {
        let mut empty_buffer = buffer.borrow_mut();
        empty_buffer.clear();
        f(&mut empty_buffer);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_vec_work_correctly() {
        let mut v: StackVec<i32, 10> = StackVec::new();
        assert_eq!(v.len(), 0);
        v.push(0);
        v.push(1);
        v.push(2);
        assert_eq!(v.len(), 3);

        let elements: Vec<_> = v.into_iter().collect();
        assert_eq!(elements.len(), 3);
        assert_eq!(elements[0], 0);
        assert_eq!(elements[1], 1);
        assert_eq!(elements[2], 2);
    }

    
    #[test]
    #[should_panic]
    fn test_stack_vec_overflow() {
        let mut v: StackVec<i32, 3> = StackVec::new();
        v.push(0);
        v.push(1);
        v.push(2);
        v.push(3);
    }

    #[test]
    fn test_use_str_buffer() {
        use_str_buffer(|buff| {
            assert!(buff.is_empty());
            buff.insert(0, 'a');
            assert_eq!(buff, "a");
        });
        use_str_buffer(|buff| {
            assert!(buff.is_empty());
        });
    }
}