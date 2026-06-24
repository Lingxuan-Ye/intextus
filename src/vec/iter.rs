use super::InlineVec;
use core::fmt;
use core::iter::FusedIterator;
use core::mem::ManuallyDrop;
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
        let index = 0;
        let buf = ManuallyDrop::new(self);
        IntoIter { index, buf }
    }
}

pub struct IntoIter<T, const N: usize> {
    index: usize,
    buf: ManuallyDrop<InlineVec<T, N>>,
}

impl<T, const N: usize> fmt::Debug for IntoIter<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = unsafe {
            self.buf
                .buf
                .get_unchecked(self.index..self.buf.len)
                .assume_init_ref()
        };
        f.debug_list().entries(slice).finish()
    }
}

impl<T, const N: usize> Clone for IntoIter<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let index = self.index;
        let mut buf = ManuallyDrop::new(InlineVec::new());
        buf.len = self.index;
        let mut iter = Self { index, buf };
        while iter.buf.len != self.buf.len {
            let index = iter.buf.len;
            unsafe {
                let value = self.buf.buf.assume_init_clone(index);
                iter.buf.buf.write(index, value);
            }
            iter.buf.len += 1;
        }
        iter
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }
        let index = self.index;
        self.index += 1;
        let value = unsafe { self.buf.buf.assume_init_read(index) };
        Some(value)
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
            let to_drop = self.index..self.buf.len;
            self.index = self.buf.len;
            unsafe {
                self.buf.buf.partial_drop(to_drop);
            }
            None
        } else {
            let index = self.index + n;
            let to_drop = self.index..index;
            self.index = index + 1;
            unsafe {
                self.buf.buf.partial_drop(to_drop);
            }
            let value = unsafe { self.buf.buf.assume_init_read(index) };
            Some(value)
        }
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;
        while self.len() != 0 {
            let index = self.index;
            self.index += 1;
            let value = unsafe { self.buf.buf.assume_init_read(index) };
            acc = f(acc, value);
        }
        acc
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    fn len(&self) -> usize {
        self.buf.len - self.index
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len() == 0 {
            return None;
        }
        self.buf.len -= 1;
        let index = self.buf.len;
        let value = unsafe { self.buf.buf.assume_init_read(index) };
        Some(value)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.len() {
            let to_drop = self.index..self.buf.len;
            self.buf.len = self.index;
            unsafe {
                self.buf.buf.partial_drop(to_drop);
            }
            None
        } else {
            let to_drop = (self.buf.len - n)..self.buf.len;
            let index = to_drop.start - 1;
            self.buf.len = index;
            unsafe {
                self.buf.buf.partial_drop(to_drop);
            }
            let value = unsafe { self.buf.buf.assume_init_read(index) };
            Some(value)
        }
    }

    fn rfold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut acc = init;
        while self.len() != 0 {
            self.buf.len -= 1;
            let index = self.buf.len;
            let value = unsafe { self.buf.buf.assume_init_read(index) };
            acc = f(acc, value);
        }
        acc
    }
}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}

impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        let to_drop = self.index..self.buf.len;
        unsafe {
            self.buf.buf.partial_drop(to_drop);
        }
    }
}
