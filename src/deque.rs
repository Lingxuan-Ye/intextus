pub use self::iter::{IntoIter, Iter, IterMut};

use crate::buf;
use crate::buf::{Buf, Span};
use crate::error::{Error, IndexOutOfBounds, UpperBound};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Bound, Index, IndexMut, RangeBounds};
use core::slice;

mod cmp;
mod convert;
mod iter;

#[cfg(feature = "serde")]
mod serde;

pub struct InlineDeque<T, const N: usize> {
    /// Invariant: `head == 0 || head < N`.
    head: usize,
    /// Invariant: `len <= N`.
    len: usize,
    buf: Buf<T, N>,
}

impl<T, const N: usize> InlineDeque<T, N> {
    pub const fn new() -> Self {
        let head = 0;
        let len = 0;
        let buf = Buf::new();
        Self { head, len, buf }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len == N
    }

    pub const fn as_slices(&self) -> (&[T], &[T]) {
        let (prefix_span, suffix_span) = self.slice_spans();
        let base = self.buf.as_ptr();
        unsafe {
            let prefix_ptr = base.add(prefix_span.start);
            let prefix = slice::from_raw_parts(prefix_ptr, prefix_span.len);
            let suffix_ptr = base.add(suffix_span.start);
            let suffix = slice::from_raw_parts(suffix_ptr, suffix_span.len);
            (prefix, suffix)
        }
    }

    pub const fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (prefix_span, suffix_span) = self.slice_spans();
        let base = self.buf.as_mut_ptr();
        unsafe {
            let prefix_ptr = base.add(prefix_span.start);
            let prefix = slice::from_raw_parts_mut(prefix_ptr, prefix_span.len);
            let suffix_ptr = base.add(suffix_span.start);
            let suffix = slice::from_raw_parts_mut(suffix_ptr, suffix_span.len);
            (prefix, suffix)
        }
    }

    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq<T>,
    {
        let (prefix, suffix) = self.as_slices();
        prefix.contains(value) || suffix.contains(value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.get_unchecked(index)) }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(self.get_unchecked_mut(index)) }
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        let index = self.physical_index(index);
        unsafe { self.buf.assume_init_ref(index) }
    }

    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        let index = self.physical_index(index);
        unsafe { self.buf.assume_init_mut(index) }
    }

    pub fn front(&self) -> Option<&T> {
        self.get(0)
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    pub fn back(&self) -> Option<&T> {
        self.get(self.len.wrapping_sub(1))
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.len.wrapping_sub(1))
    }

    pub fn push_front(&mut self, value: T) -> Result<(), Error<T>> {
        self.push_front_mut(value).map(|_| ())
    }

    pub fn push_front_mut(&mut self, value: T) -> Result<&mut T, Error<T>> {
        if self.len == N {
            return Err(Error::capacity_overflow().with_value(value));
        }
        let index = Buf::<T, N>::wrap_sub(self.head, 1);
        let slot = unsafe { self.buf.write(index, value) };
        self.head = index;
        self.len += 1;
        Ok(slot)
    }

    pub fn push_back(&mut self, value: T) -> Result<(), Error<T>> {
        self.push_back_mut(value).map(|_| ())
    }

    pub fn push_back_mut(&mut self, value: T) -> Result<&mut T, Error<T>> {
        if self.len == N {
            return Err(Error::capacity_overflow().with_value(value));
        }
        let index = Buf::<T, N>::wrap_add(self.head, self.len);
        let slot = unsafe { self.buf.write(index, value) };
        self.len += 1;
        Ok(slot)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let index = self.head;
        self.head = Buf::<T, N>::wrap_add(self.head, 1);
        self.len -= 1;
        unsafe { Some(self.buf.assume_init_read(index)) }
    }

    pub fn pop_front_if<F>(&mut self, predicate: F) -> Option<T>
    where
        F: FnOnce(&mut T) -> bool,
    {
        let front = self.front_mut()?;
        if predicate(front) {
            self.pop_front()
        } else {
            None
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let index = Buf::<T, N>::wrap_add(self.head, self.len);
        unsafe { Some(self.buf.assume_init_read(index)) }
    }

    pub fn pop_back_if<F>(&mut self, predicate: F) -> Option<T>
    where
        F: FnOnce(&mut T) -> bool,
    {
        let back = self.back_mut()?;
        if predicate(back) {
            self.pop_back()
        } else {
            None
        }
    }

    pub fn insert(&mut self, index: usize, value: T) -> Result<(), Error<T>> {
        self.insert_mut(index, value).map(|_| ())
    }

    pub fn insert_mut(&mut self, index: usize, value: T) -> Result<&mut T, Error<T>> {
        if index > self.len {
            let upper = UpperBound::Included(self.len);
            return Err(Error::index_out_of_bounds(index, upper).with_value(value));
        }
        if self.len == N {
            return Err(Error::capacity_overflow().with_value(value));
        }
        let prefix_len = index;
        let suffix_len = self.len - prefix_len;
        if prefix_len <= suffix_len {
            let old_head = self.head;
            self.head = Buf::<T, N>::wrap_sub(self.head, 1);
            unsafe {
                self.buf.wrap_copy_within(old_head, self.head, prefix_len);
            }
            let index = self.physical_index(index);
            let slot = unsafe { self.buf.write(index, value) };
            self.len += 1;
            Ok(slot)
        } else {
            let index = self.physical_index(index);
            let next = Buf::<T, N>::wrap_add(index, 1);
            unsafe {
                self.buf.wrap_copy_within(index, next, suffix_len);
            }
            let slot = unsafe { self.buf.write(index, value) };
            self.len += 1;
            Ok(slot)
        }
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        let prefix_len = index;
        let suffix_len = self.len - prefix_len - 1;
        let index = self.physical_index(index);
        self.len -= 1;
        let value = unsafe { self.buf.assume_init_read(index) };
        if prefix_len <= suffix_len {
            let old_head = self.head;
            self.head = Buf::<T, N>::wrap_add(self.head, 1);
            unsafe {
                self.buf.wrap_copy_within(old_head, self.head, prefix_len);
            }
        } else {
            let next = Buf::<T, N>::wrap_add(index, 1);
            unsafe {
                self.buf.wrap_copy_within(next, index, suffix_len);
            }
        }
        Some(value)
    }

    pub fn swap_remove_front(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        let index = self.physical_index(index);
        let front = self.head;
        self.head = Buf::<T, N>::wrap_add(self.head, 1);
        self.len -= 1;
        let value = unsafe { self.buf.assume_init_read(index) };
        if index != front {
            unsafe {
                self.buf.copy_within_nonoverlapping(front, index, 1);
            }
        }
        Some(value)
    }

    pub fn swap_remove_back(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        let index = self.physical_index(index);
        self.len -= 1;
        let back = Buf::<T, N>::wrap_add(self.head, self.len);
        let value = unsafe { self.buf.assume_init_read(index) };
        if index != back {
            unsafe {
                self.buf.copy_within_nonoverlapping(back, index, 1);
            }
        }
        Some(value)
    }

    pub const fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        if i >= self.len {
            let upper = UpperBound::Excluded(self.len);
            return Err(Error::index_out_of_bounds(i, upper));
        }
        if j >= self.len {
            let upper = UpperBound::Excluded(self.len);
            return Err(Error::index_out_of_bounds(j, upper));
        }
        let i = self.physical_index(i);
        let j = self.physical_index(j);
        unsafe {
            self.buf.swap(i, j);
        }
        Ok(())
    }

    pub fn extend<I>(&mut self, iter: I) -> I::IntoIter
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let head_to_end = N - self.head;
        if self.len < head_to_end {
            let tail = self.head + self.len;
            for (index, value) in (tail..N).zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            for (index, value) in (0..self.head).zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        } else {
            let tail = self.len - head_to_end;
            for (index, value) in (tail..self.head).zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        }
        iter
    }

    pub fn extend_front<I>(&mut self, iter: I) -> I::IntoIter
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        let head_to_end = N - self.head;
        if self.len < head_to_end {
            let tail = self.head + self.len;
            for (index, value) in (0..self.head).rev().zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
            for (index, value) in (tail..N).rev().zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
        } else {
            let tail = self.len - head_to_end;
            for (index, value) in (tail..self.head).rev().zip(&mut iter) {
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
        }
        iter
    }

    pub fn extend_from_slice<'a>(&mut self, slice: &'a [T]) -> &'a [T]
    where
        T: Clone,
    {
        let mut iter = slice.iter();
        let head_to_end = N - self.head;
        if self.len < head_to_end {
            let tail = self.head + self.len;
            for (index, value) in (tail..N).zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            for (index, value) in (0..self.head).zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        } else {
            let tail = self.len - head_to_end;
            for (index, value) in (tail..self.head).zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        }
        iter.as_slice()
    }

    pub fn extend_front_from_slice<'a>(&mut self, slice: &'a [T]) -> &'a [T]
    where
        T: Clone,
    {
        let mut iter = slice.iter();
        let head_to_end = N - self.head;
        if self.len < head_to_end {
            let tail = self.head + self.len;
            for (index, value) in (0..self.head).rev().zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
            for (index, value) in (tail..N).rev().zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
        } else {
            let tail = self.len - head_to_end;
            for (index, value) in (tail..self.head).rev().zip(&mut iter) {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.head = index;
                self.len += 1;
            }
        }
        iter.as_slice()
    }

    pub fn resize(&mut self, len: usize, value: T) -> Result<Option<T>, Error<T>>
    where
        T: Clone,
    {
        if len > N {
            return Err(Error::capacity_overflow().with_value(value));
        }
        if len <= self.len {
            self.truncate(len);
            return Ok(Some(value));
        }
        let head_to_end = N - self.head;
        if len <= head_to_end {
            let tail = self.head + self.len;
            let back = self.head + len - 1;
            for index in tail..back {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            unsafe {
                self.buf.write(back, value);
            }
            self.len += 1;
        } else if self.len < head_to_end {
            let tail = self.head + self.len;
            let back = len - head_to_end - 1;
            for index in tail..N {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            for index in 0..back {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            unsafe {
                self.buf.write(back, value);
            }
            self.len += 1;
        } else {
            let tail = self.len - head_to_end;
            let back = len - head_to_end - 1;
            for index in tail..back {
                let value = value.clone();
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            unsafe {
                self.buf.write(back, value);
            }
            self.len += 1;
        }
        Ok(None)
    }

    pub fn resize_with<F>(&mut self, len: usize, mut f: F) -> Result<(), Error>
    where
        F: FnMut(usize) -> T,
    {
        if len > N {
            return Err(Error::capacity_overflow());
        }
        if len <= self.len {
            self.truncate(len);
            return Ok(());
        }
        let head_to_end = N - self.head;
        if len <= head_to_end {
            for index in self.len..len {
                let value = f(index);
                let index = self.head + index;
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        } else if self.len < head_to_end {
            for index in self.len..head_to_end {
                let value = f(index);
                let index = self.head + index;
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
            for index in head_to_end..len {
                let value = f(index);
                let index = index - head_to_end;
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        } else {
            for index in self.len..len {
                let value = f(index);
                let index = index - head_to_end;
                unsafe {
                    self.buf.write(index, value);
                }
                self.len += 1;
            }
        }
        Ok(())
    }

    pub fn truncate(&mut self, len: usize) {
        if len >= self.len {
            return;
        }
        let head_to_end = N - self.head;
        if self.len <= head_to_end {
            let to_drop = (self.head + len)..(self.head + self.len);
            self.len = len;
            unsafe {
                self.buf.assume_init_drop(to_drop);
            }
        } else if len < head_to_end {
            let prefix_to_drop = (self.head + len)..N;
            let suffix_to_drop = 0..(self.len - head_to_end);
            self.len = len;
            unsafe {
                self.buf.assume_init_drop(prefix_to_drop);
                self.buf.assume_init_drop(suffix_to_drop);
            }
        } else {
            let to_drop = (len - head_to_end)..(self.len - head_to_end);
            self.len = len;
            unsafe {
                self.buf.assume_init_drop(to_drop);
            }
        }
    }

    pub fn truncate_front(&mut self, len: usize) {
        if len >= self.len {
            return;
        }
        let drop_len = self.len - len;
        let head_to_end = N - self.head;
        if drop_len < head_to_end {
            let old_head = self.head;
            self.head += drop_len;
            self.len = len;
            let to_drop = old_head..self.head;
            unsafe {
                self.buf.assume_init_drop(to_drop);
            }
        } else if drop_len == head_to_end {
            let old_head = self.head;
            self.head = 0;
            self.len = len;
            let to_drop = old_head..N;
            unsafe {
                self.buf.assume_init_drop(to_drop);
            }
        } else {
            let old_head = self.head;
            self.head = drop_len - head_to_end;
            self.len = len;
            let prefix_to_drop = old_head..N;
            let suffix_to_drop = 0..self.head;
            unsafe {
                self.buf.assume_init_drop(prefix_to_drop);
                self.buf.assume_init_drop(suffix_to_drop);
            }
        }
    }

    pub const fn split_off(&mut self, at: usize) -> Result<Self, Error> {
        if at > self.len {
            let upper = UpperBound::Excluded(self.len);
            return Err(Error::index_out_of_bounds(at, upper));
        }
        let mut result = Self::new();
        let src_len = self.len;
        let dst_len = self.len - at;
        self.len = at;
        let head_to_end = N - self.head;
        if src_len <= head_to_end {
            unsafe {
                let src_index = self.head + at;
                let dst_index = 0;
                let count = dst_len;
                buf::copy_nonoverlapping(&self.buf, &mut result.buf, src_index, dst_index, count);
            }
        } else if at < head_to_end {
            unsafe {
                let src_index = self.head + at;
                let dst_index = 0;
                let count = head_to_end - at;
                buf::copy_nonoverlapping(&self.buf, &mut result.buf, src_index, dst_index, count);
                let src_index = 0;
                let dst_index = count;
                let count = dst_len - count;
                buf::copy_nonoverlapping(&self.buf, &mut result.buf, src_index, dst_index, count);
            }
        } else {
            unsafe {
                let src_index = at - head_to_end;
                let dst_index = 0;
                let count = dst_len;
                buf::copy_nonoverlapping(&self.buf, &mut result.buf, src_index, dst_index, count);
            }
        }
        result.len = dst_len;
        Ok(result)
    }

    pub const fn rotate_left(&mut self, n: usize) {
        if self.len == 0 {
            return;
        }
        let n = n % self.len;
        let prefix_len = n;
        let suffix_len = self.len - n;
        if prefix_len <= suffix_len {
            let tail = Buf::<T, N>::wrap_add(self.head, self.len);
            unsafe {
                self.buf.wrap_copy_within(self.head, tail, prefix_len);
            }
            self.head = Buf::<T, N>::wrap_add(self.head, prefix_len);
        } else {
            self.head = Buf::<T, N>::wrap_sub(self.head, suffix_len);
            let tail = Buf::<T, N>::wrap_add(self.head, self.len);
            unsafe {
                self.buf.wrap_copy_within(tail, self.head, suffix_len);
            }
        }
    }

    pub const fn rotate_right(&mut self, n: usize) {
        if self.len == 0 {
            return;
        }
        let n = n % self.len;
        let prefix_len = self.len - n;
        let suffix_len = n;
        if suffix_len <= prefix_len {
            self.head = Buf::<T, N>::wrap_sub(self.head, suffix_len);
            let tail = Buf::<T, N>::wrap_add(self.head, self.len);
            unsafe {
                self.buf.wrap_copy_within(tail, self.head, suffix_len);
            }
        } else {
            let tail = Buf::<T, N>::wrap_add(self.head, self.len);
            unsafe {
                self.buf.wrap_copy_within(self.head, tail, prefix_len);
            }
            self.head = Buf::<T, N>::wrap_add(self.head, prefix_len);
        }
    }

    pub fn binary_search(&self, value: &T) -> Result<usize, usize>
    where
        T: Ord,
    {
        self.binary_search_by(|element| element.cmp(value))
    }

    pub fn binary_search_by<F>(&self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&T) -> Ordering,
    {
        let (prefix, suffix) = self.as_slices();
        let prefix_len = prefix.len();
        match suffix.first().map(&mut f) {
            Some(Ordering::Less) => match suffix.binary_search_by(f) {
                Ok(index) => Ok(prefix_len + index),
                Err(index) => Err(prefix_len + index),
            },
            Some(Ordering::Equal) => Ok(prefix_len),
            _ => prefix.binary_search_by(f),
        }
    }

    pub fn binary_search_by_key<K, F>(&self, key: &K, mut f: F) -> Result<usize, usize>
    where
        K: Ord,
        F: FnMut(&T) -> K,
    {
        self.binary_search_by(|element| f(element).cmp(key))
    }

    pub const fn make_contiguous(&mut self) -> &mut [T] {
        if size_of::<T>() == 0 {
            self.head = 0;
        }
        let base = self.buf.as_mut_ptr();
        let head_to_end = N - self.head;
        if self.len <= head_to_end {
            unsafe {
                let ptr = base.add(self.head);
                return slice::from_raw_parts_mut(ptr, self.len);
            }
        }
        let free = N - self.len;
        let tail = self.len - head_to_end;
        let prefix_len = head_to_end;
        let suffix_len = tail;
        unsafe {
            if free >= prefix_len || free < suffix_len {
                self.buf.wrap_copy_within(self.head, 0, self.len);
                self.head = 0;
            } else {
                self.buf.wrap_copy_within(self.head, tail, self.len);
                self.head = tail;
            }
            let ptr = base.add(self.head);
            slice::from_raw_parts_mut(ptr, self.len)
        }
    }

    pub fn clear(&mut self) {
        let (prefix_to_drop, suffix_to_drop) = self.slice_spans();
        self.head = 0;
        self.len = 0;
        unsafe {
            self.buf.assume_init_drop(prefix_to_drop);
            self.buf.assume_init_drop(suffix_to_drop);
        }
    }

    pub(crate) const fn buf(&self) -> &Buf<T, N> {
        &self.buf
    }

    pub(crate) const fn slice_spans(&self) -> (Span, Span) {
        let prefix;
        let suffix;
        let head_to_end = N - self.head;
        if self.len <= head_to_end {
            prefix = Span {
                start: self.head,
                len: self.len,
            };
            suffix = Span { start: 0, len: 0 };
        } else {
            prefix = Span {
                start: self.head,
                len: head_to_end,
            };
            suffix = Span {
                start: 0,
                len: self.len - head_to_end,
            };
        }
        (prefix, suffix)
    }

    /// The caller must ensure that:
    ///
    /// - `index <= N`.
    const fn physical_index(&self, index: usize) -> usize {
        Buf::<T, N>::wrap_add(self.head, index)
    }

    fn physical_spans<R>(&self, range: R) -> Option<(Span, Span)>
    where
        R: RangeBounds<usize>,
    {
        let end = match range.end_bound() {
            Bound::Included(&end) if end >= self.len => return None,
            Bound::Included(&end) => end + 1,
            Bound::Excluded(&end) if end > self.len => return None,
            Bound::Excluded(&end) => end,
            Bound::Unbounded => self.len,
        };
        let start = match range.start_bound() {
            Bound::Included(&start) if start > end => return None,
            Bound::Included(&start) => start,
            Bound::Excluded(&start) if start >= end => return None,
            Bound::Excluded(&start) => start + 1,
            Bound::Unbounded => 0,
        };
        let range = start..end;
        let head_to_end = N - self.head;
        let prefix;
        let suffix;
        if end <= head_to_end {
            prefix = Span {
                start: self.head + range.start,
                len: range.end - range.start,
            };
            suffix = Span { start: 0, len: 0 };
        } else if start < head_to_end {
            prefix = Span {
                start: self.head + range.start,
                len: head_to_end - range.start,
            };
            suffix = Span {
                start: 0,
                len: range.end - head_to_end,
            };
        } else {
            prefix = Span { start: N, len: 0 };
            suffix = Span {
                start: range.start - head_to_end,
                len: range.end - range.start,
            };
        }
        Some((prefix, suffix))
    }
}

impl<T, const N: usize> fmt::Debug for InlineDeque<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, suffix) = self.as_slices();
        f.debug_list().entries(prefix).entries(suffix).finish()
    }
}

impl<T, const N: usize> Default for InlineDeque<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Clone for InlineDeque<T, N>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut result = Self::new();
        for (index, value) in self.iter().enumerate() {
            let value = value.clone();
            unsafe {
                result.buf.write(index, value);
            }
            result.len += 1;
        }
        result
    }
}

impl<T, const N: usize> Hash for InlineDeque<T, N>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let (prefix, suffix) = self.as_slices();
        prefix.hash(state);
        suffix.hash(state);
    }
}

impl<T, const N: usize> Index<usize> for InlineDeque<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let len = self.len;
        match self.get(index) {
            None => {
                let upper = UpperBound::Excluded(len);
                let error = IndexOutOfBounds::new(index, upper);
                panic!("{error}")
            }
            Some(output) => output,
        }
    }
}

impl<T, const N: usize> IndexMut<usize> for InlineDeque<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let len = self.len;
        match self.get_mut(index) {
            None => {
                let upper = UpperBound::Excluded(len);
                let error = IndexOutOfBounds::new(index, upper);
                panic!("{error}")
            }
            Some(output) => output,
        }
    }
}

impl<T, const N: usize> Drop for InlineDeque<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}
