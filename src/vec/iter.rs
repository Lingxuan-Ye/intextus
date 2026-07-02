use super::InlineVec;
use crate::buf::Buf;
use core::fmt;
use core::iter::FusedIterator;
use core::mem::ManuallyDrop;
use core::range::Range;
use core::slice::{Iter, IterMut};

impl<'a, T, const N: usize> IntoIterator for &'a InlineVec<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut InlineVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> IntoIterator for InlineVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        let start = 0;
        let end_and_buf = ManuallyDrop::new(self);
        IntoIter { start, end_and_buf }
    }
}

pub struct IntoIter<T, const N: usize> {
    start: usize,
    end_and_buf: ManuallyDrop<InlineVec<T, N>>,
}

impl<T, const N: usize> IntoIter<T, N> {
    pub fn as_slice(&self) -> &[T] {
        let alive = self.alive();
        unsafe {
            self.buf()
                .as_uninit_array()
                .get_unchecked(alive)
                .assume_init_ref()
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let alive = self.alive();
        unsafe {
            self.buf_mut()
                .as_uninit_array_mut()
                .get_unchecked_mut(alive)
                .assume_init_mut()
        }
    }

    fn alive(&self) -> Range<usize> {
        let start = self.start();
        let end = self.end();
        Range { start, end }
    }

    fn start(&self) -> usize {
        self.start
    }

    unsafe fn start_mut(&mut self) -> &mut usize {
        &mut self.start
    }

    fn end(&self) -> usize {
        self.end_and_buf.len
    }

    unsafe fn end_mut(&mut self) -> &mut usize {
        &mut self.end_and_buf.len
    }

    fn buf(&self) -> &Buf<T, N> {
        &self.end_and_buf.buf
    }

    fn buf_mut(&mut self) -> &mut Buf<T, N> {
        &mut self.end_and_buf.buf
    }
}

impl<T, const N: usize> fmt::Debug for IntoIter<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = self.as_slice();
        f.debug_tuple("IntoIter").field(&slice).finish()
    }
}

impl<T, const N: usize> Clone for IntoIter<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let start = 0;
        let end_and_buf = ManuallyDrop::new(InlineVec::new());
        let mut result = Self { start, end_and_buf };
        unsafe {
            *result.start_mut() = self.start();
            *result.end_mut() = self.start();
        }
        for index in self.alive() {
            unsafe {
                let value = self.buf().assume_init_ref(index).clone();
                result.buf_mut().write(index, value);
                *result.end_mut() += 1;
            }
        }
        result
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start() == self.end() {
            return None;
        }
        let index = self.start();
        unsafe {
            *self.start_mut() += 1;
            Some(self.buf().assume_init_read(index))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            let to_drop = self.alive();
            unsafe {
                *self.start_mut() = self.end();
                self.buf_mut().assume_init_drop(to_drop);
            }
            return None;
        }
        let index = self.start() + n;
        let to_drop = self.start()..index;
        unsafe {
            *self.start_mut() = index + 1;
            self.buf_mut().assume_init_drop(to_drop);
            Some(self.buf().assume_init_read(index))
        }
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut accum = init;
        for index in self.alive() {
            unsafe {
                *self.start_mut() += 1;
                let value = self.buf().assume_init_read(index);
                accum = f(accum, value);
            }
        }
        accum
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    fn len(&self) -> usize {
        self.end() - self.start()
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start() == self.end() {
            return None;
        }
        unsafe {
            *self.end_mut() -= 1;
        }
        let index = self.end();
        unsafe { Some(self.buf().assume_init_read(index)) }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            let to_drop = self.alive();
            unsafe {
                *self.end_mut() = self.start();
                self.buf_mut().assume_init_drop(to_drop);
            }
            return None;
        }
        let to_drop = (self.end() - n)..self.end();
        let index = to_drop.start - 1;
        unsafe {
            *self.end_mut() = index;
            self.buf_mut().assume_init_drop(to_drop);
            Some(self.buf().assume_init_read(index))
        }
    }

    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut accum = init;
        for index in self.alive().into_iter().rev() {
            unsafe {
                *self.end_mut() -= 1;
                let value = self.buf().assume_init_read(index);
                accum = f(accum, value);
            }
        }
        accum
    }
}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        let to_drop = self.alive();
        unsafe {
            self.buf_mut().assume_init_drop(to_drop);
        }
    }
}
