use num::{cast::AsPrimitive, Integer};
use std::{
    clone::Clone,
    ops::{Index, IndexMut},
};

/// A simple ring buffer data structure
/// The buffer is always full and wraps around so that the oldest elements are overwritten.
/// It supports both negative and positive indexing and also indexing past the size.
pub struct RingBuffer<T> {
    elements: Vec<T>,
    current_index: isize,
}

pub struct RingBufferIter<'a, T> {
    ring_buffer: &'a RingBuffer<T>,
    index: usize,
}

pub struct RingBufferIterMut<'a, T> {
    ring_buffer: &'a mut RingBuffer<T>,
    index: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(size: usize, default_value: T) -> Self {
        let mut elements = Vec::new();
        elements.resize(size, default_value);
        Self {
            current_index: 0,
            elements,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn iter(&self) -> RingBufferIter<'_, T> {
        RingBufferIter {
            ring_buffer: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> RingBufferIterMut<'_, T> {
        RingBufferIterMut {
            ring_buffer: self,
            index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn resize(&mut self, new_size: usize, default_value: T) {
        let index = self.get_array_index(0);
        self.elements.rotate_left(index);
        self.elements.resize(new_size, default_value);
        self.current_index = 0;
    }

    pub fn rotate(&mut self, num: isize) {
        self.current_index += num;
    }

    fn get_array_index(&self, index: isize) -> usize {
        let num = self.elements.len() as isize;
        (self.current_index + index).rem_euclid(num) as usize
    }
}

impl<T: Clone, I: Integer + AsPrimitive<isize>> Index<I> for RingBuffer<T> {
    type Output = T;

    fn index(&self, index: I) -> &Self::Output {
        let array_index = self.get_array_index(index.as_());
        &self.elements[array_index]
    }
}

impl<T: Clone, I: Integer + AsPrimitive<isize>> IndexMut<I> for RingBuffer<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let array_index = self.get_array_index(index.as_());
        &mut self.elements[array_index]
    }
}

impl<'a, T: Clone> Iterator for RingBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ring_buffer.elements.len() {
            return None;
        }

        let ret = &self.ring_buffer[self.index];
        self.index += 1;
        Some(ret)
    }
}

impl<'a, T: Clone> Iterator for RingBufferIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.ring_buffer.elements.len() {
            return None;
        }

        let elements = self.ring_buffer.elements.as_mut_ptr();
        let array_index = self.ring_buffer.get_array_index(self.index as isize);
        let ret = unsafe { &mut *elements.add(array_index) };
        self.index += 1;
        Some(ret)
    }
}

impl<'a, T: Clone> IntoIterator for &'a RingBuffer<T> {
    type Item = &'a T;

    type IntoIter = RingBufferIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Clone> IntoIterator for &'a mut RingBuffer<T> {
    type Item = &'a mut T;

    type IntoIter = RingBufferIterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let buffer = RingBuffer::<i32>::new(0, 5);
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.is_empty(), true);
    }

    #[test]
    fn single_element() {
        let mut buffer = RingBuffer::<i32>::new(1, 5);
        assert_eq!(buffer.len(), 1);
        assert_eq!(buffer.is_empty(), false);
        assert_eq!(buffer[0], 5);
        buffer[0] = 3;
        assert_eq!(buffer[0], 3);
        assert_eq!(buffer.iter().eq([3].iter()), true);
        buffer.iter_mut().zip(&[7]).for_each(|(a, b)| *a = *b);
        assert_eq!(buffer.iter().eq([7].iter()), true);
    }

    #[test]
    fn three_elements() {
        let mut buffer = RingBuffer::<i32>::new(3, 0);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer[0], 0);
        assert_eq!(buffer[2], 0);
        buffer.iter_mut().zip(&[1, 2, 3]).for_each(|(a, b)| *a = *b);
        assert_eq!(buffer.iter().eq([1, 2, 3].iter()), true);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[2], 3);
    }

    #[test]
    fn rotate_forwards() {
        let mut buffer = RingBuffer::<i32>::new(5, 0);
        buffer
            .iter_mut()
            .zip(&[1, 2, 3, 4, 5])
            .for_each(|(a, b)| *a = *b);
        buffer.rotate(2);
        assert_eq!(buffer[0], 3);
        assert_eq!(buffer[4], 2);
        assert_eq!(buffer.iter().eq([3, 4, 5, 1, 2].iter()), true);
        assert_eq!(buffer[-2], 1);
        buffer
            .iter_mut()
            .zip(&[5, 6, 7, 8, 9])
            .for_each(|(a, b)| *a = *b);
        assert_eq!(buffer.iter().eq([5, 6, 7, 8, 9].iter()), true);
    }

    #[test]
    fn rotate_backwards() {
        let mut buffer = RingBuffer::<i32>::new(3, 0);
        assert_eq!(buffer.iter().eq([0, 0, 0].iter()), true);
        buffer[0] = 0;
        buffer[1] = 2;
        buffer[2] = 5;
        buffer.rotate(-1);
        assert_eq!(buffer.iter().eq([5, 0, 2].iter()), true);
        assert_eq!(buffer[-1], 2);
        buffer.iter_mut().zip(&[5, 6, 7]).for_each(|(a, b)| *a = *b);
        assert_eq!(buffer.iter().eq([5, 6, 7].iter()), true);
    }

    #[test]
    fn resize_bigger() {
        let mut buffer = RingBuffer::<i32>::new(3, 0);
        assert_eq!(buffer.iter().eq([0, 0, 0].iter()), true);
        buffer[0] = 0;
        buffer[1] = 2;
        buffer[2] = 5;
        buffer.rotate(-1);
        buffer.resize(5, 7);
        assert_eq!(buffer.iter().eq([5, 0, 2, 7, 7].iter()), true);
    }

    #[test]
    fn resize_smaller() {
        let mut buffer = RingBuffer::<i32>::new(3, 0);
        assert_eq!(buffer.iter().eq([0, 0, 0].iter()), true);
        buffer[0] = 0;
        buffer[1] = 2;
        buffer[2] = 5;
        buffer.rotate(1);
        buffer.resize(2, 7);
        assert_eq!(buffer.iter().eq([2, 5].iter()), true);
    }
}
