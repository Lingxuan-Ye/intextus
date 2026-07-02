use super::InlineDeque;
use core::fmt;
use core::iter::FusedIterator;
use core::mem;
use core::ops::RangeBounds;
use core::range::Range;
use core::slice;

impl<T, const N: usize> InlineDeque<T, N> {
    pub fn iter(&self) -> Iter<'_, T> {
        let (prefix, suffix) = self.as_slices();
        let prefix = prefix.iter();
        let suffix = suffix.iter();
        Iter { prefix, suffix }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        let (prefix, suffix) = self.as_mut_slices();
        let prefix = prefix.iter_mut();
        let suffix = suffix.iter_mut();
        IterMut { prefix, suffix }
    }

    pub fn range<R>(&self, range: R) -> Option<Iter<'_, T>>
    where
        R: RangeBounds<usize>,
    {
        let (prefix_span, suffix_span) = self.physical_spans(range)?;
        let base = self.buf.as_ptr();
        unsafe {
            let prefix_ptr = base.add(prefix_span.start);
            let prefix = slice::from_raw_parts(prefix_ptr, prefix_span.len).iter();
            let suffix_ptr = base.add(suffix_span.start);
            let suffix = slice::from_raw_parts(suffix_ptr, suffix_span.len).iter();
            Some(Iter { prefix, suffix })
        }
    }

    pub fn range_mut<R>(&mut self, range: R) -> Option<IterMut<'_, T>>
    where
        R: RangeBounds<usize>,
    {
        let (prefix_span, suffix_span) = self.physical_spans(range)?;
        let base = self.buf.as_mut_ptr();
        unsafe {
            let prefix_ptr = base.add(prefix_span.start);
            let prefix = slice::from_raw_parts_mut(prefix_ptr, prefix_span.len).iter_mut();
            let suffix_ptr = base.add(suffix_span.start);
            let suffix = slice::from_raw_parts_mut(suffix_ptr, suffix_span.len).iter_mut();
            Some(IterMut { prefix, suffix })
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a InlineDeque<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut InlineDeque<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T, const N: usize> IntoIterator for InlineDeque<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { deque: self }
    }
}

pub struct Iter<'a, T> {
    prefix: slice::Iter<'a, T>,
    suffix: slice::Iter<'a, T>,
}

impl<T> Iter<'_, T> {
    pub fn as_slices(&self) -> (&[T], &[T]) {
        (self.prefix.as_slice(), self.suffix.as_slice())
    }
}

impl<T> fmt::Debug for Iter<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, suffix) = self.as_slices();
        f.debug_tuple("Iter").field(&prefix).field(&suffix).finish()
    }
}

impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        let prefix = Default::default();
        let suffix = Default::default();
        Self { prefix, suffix }
    }
}

impl<T> Clone for Iter<'_, T> {
    fn clone(&self) -> Self {
        let prefix = self.prefix.clone();
        let suffix = self.suffix.clone();
        Self { prefix, suffix }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.prefix.next() {
            Some(item)
        } else {
            mem::swap(&mut self.prefix, &mut self.suffix);
            self.prefix.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let accum = self.prefix.fold(init, &mut f);
        self.suffix.fold(accum, f)
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.prefix.len() + self.suffix.len()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.suffix.next_back() {
            Some(item)
        } else {
            mem::swap(&mut self.prefix, &mut self.suffix);
            self.suffix.next_back()
        }
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let accum = self.suffix.rfold(init, &mut f);
        self.prefix.rfold(accum, f)
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

pub struct IterMut<'a, T> {
    prefix: slice::IterMut<'a, T>,
    suffix: slice::IterMut<'a, T>,
}

impl<T> IterMut<'_, T> {
    pub fn as_slices(&self) -> (&[T], &[T]) {
        (self.prefix.as_slice(), self.suffix.as_slice())
    }
}

impl<T> fmt::Debug for IterMut<'_, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, suffix) = self.as_slices();
        f.debug_tuple("IterMut")
            .field(&prefix)
            .field(&suffix)
            .finish()
    }
}

impl<T> Default for IterMut<'_, T> {
    fn default() -> Self {
        let prefix = Default::default();
        let suffix = Default::default();
        Self { prefix, suffix }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.prefix.next() {
            Some(item)
        } else {
            mem::swap(&mut self.prefix, &mut self.suffix);
            self.prefix.next()
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let accum = self.prefix.fold(init, &mut f);
        self.suffix.fold(accum, f)
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        self.prefix.len() + self.suffix.len()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.suffix.next_back() {
            Some(item)
        } else {
            mem::swap(&mut self.prefix, &mut self.suffix);
            self.suffix.next_back()
        }
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let accum = self.suffix.rfold(init, &mut f);
        self.prefix.rfold(accum, f)
    }
}

impl<T> FusedIterator for IterMut<'_, T> {}

#[derive(Clone)]
pub struct IntoIter<T, const N: usize> {
    deque: InlineDeque<T, N>,
}

impl<T, const N: usize> IntoIter<T, N> {
    pub const fn as_slices(&self) -> (&[T], &[T]) {
        self.deque.as_slices()
    }

    pub const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        self.deque.as_mut_slices()
    }
}

impl<T, const N: usize> fmt::Debug for IntoIter<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, suffix) = self.as_slices();
        f.debug_tuple("IntoIter")
            .field(&prefix)
            .field(&suffix)
            .finish()
    }
}

impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.deque.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut deque = self.deque;
        let (prefix, suffix) = deque.slice_spans();
        deque.head = 0;
        deque.len = 0;
        let mut accum = init;
        for index in Range::from(prefix) {
            unsafe {
                let value = deque.buf.assume_init_read(index);
                accum = f(accum, value);
            }
        }
        for index in Range::from(suffix) {
            unsafe {
                let value = deque.buf.assume_init_read(index);
                accum = f(accum, value);
            }
        }
        accum
    }
}

impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {
    fn len(&self) -> usize {
        self.deque.len()
    }
}

impl<T, const N: usize> DoubleEndedIterator for IntoIter<T, N> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.deque.pop_back()
    }

    fn rfold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        let mut deque = self.deque;
        let (prefix, suffix) = deque.slice_spans();
        deque.head = 0;
        deque.len = 0;
        let mut accum = init;
        for index in Range::from(suffix).into_iter().rev() {
            unsafe {
                let value = deque.buf.assume_init_read(index);
                accum = f(accum, value);
            }
        }
        for index in Range::from(prefix).into_iter().rev() {
            unsafe {
                let value = deque.buf.assume_init_read(index);
                accum = f(accum, value);
            }
        }
        accum
    }
}

impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}
