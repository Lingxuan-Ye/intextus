mod convert;
mod iter;

pub use self::iter::IntoIter;

use crate::buf;
use crate::buf::Buf;
use crate::deque::InlineDeque;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice;
use core::slice::SliceIndex;

pub struct InlineVec<T, const N: usize> {
    len: usize,
    buf: Buf<T, N>,
}

impl<T, const N: usize> InlineVec<T, N> {
    pub const fn new() -> Self {
        let len = 0;
        let buf = Buf::new();
        Self { len, buf }
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

    pub const fn as_ptr(&self) -> *const T {
        self.buf.as_ptr()
    }

    pub const fn as_mut_ptr(&mut self) -> *mut T {
        self.buf.as_mut_ptr()
    }

    pub const fn as_slice(&self) -> &[T] {
        let base = self.as_ptr();
        unsafe { slice::from_raw_parts(base, self.len) }
    }

    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        let base = self.as_mut_ptr();
        unsafe { slice::from_raw_parts_mut(base, self.len) }
    }

    pub const unsafe fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    pub fn push(&mut self, value: T) -> Option<()> {
        self.push_mut(value).map(|_| ())
    }

    pub fn push_mut(&mut self, value: T) -> Option<&mut T> {
        if self.len == N {
            return None;
        }
        let index = self.len;
        let slot = unsafe { self.buf.write(index, value) };
        self.len += 1;
        Some(slot)
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        let index = self.len;
        unsafe { Some(self.buf.assume_init_read(index)) }
    }

    pub fn pop_if<F>(&mut self, predicate: F) -> Option<T>
    where
        F: FnOnce(&mut T) -> bool,
    {
        let last = self.last_mut()?;
        if predicate(last) { self.pop() } else { None }
    }

    pub fn insert(&mut self, index: usize, value: T) -> Option<()> {
        self.insert_mut(index, value).map(|_| ())
    }

    pub fn insert_mut(&mut self, index: usize, value: T) -> Option<&mut T> {
        if index > self.len || self.len == N {
            return None;
        }
        if index != self.len {
            unsafe {
                self.buf.copy_within(index, index + 1, self.len - index);
            }
        }
        let slot = unsafe { self.buf.write(index, value) };
        self.len += 1;
        Some(slot)
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        self.len -= 1;
        let value = unsafe { self.buf.assume_init_read(index) };
        if index != self.len {
            unsafe {
                self.buf.copy_within(index + 1, index, self.len - index);
            }
        }
        Some(value)
    }

    pub fn swap_remove(&mut self, index: usize) -> Option<T> {
        if index >= self.len {
            return None;
        }
        self.len -= 1;
        let value = unsafe { self.buf.assume_init_read(index) };
        if index != self.len {
            unsafe {
                self.buf.copy_within_nonoverlapping(self.len, index, 1);
            }
        }
        Some(value)
    }

    pub const fn swap(&mut self, i: usize, j: usize) -> Option<()> {
        if i >= self.len || j >= self.len {
            return None;
        }
        unsafe {
            self.buf.swap(i, j);
        }
        Some(())
    }

    pub fn extend<I>(&mut self, iter: I) -> I::IntoIter
    where
        I: IntoIterator<Item = T>,
    {
        let mut iter = iter.into_iter();
        for (index, value) in (self.len..N).zip(&mut iter) {
            unsafe {
                self.buf.write(index, value);
            }
            self.len += 1;
        }
        iter
    }

    pub fn extend_from_slice<'a>(&mut self, slice: &'a [T]) -> &'a [T]
    where
        T: Clone,
    {
        let mut iter = slice.iter();
        for (index, value) in (self.len..N).zip(&mut iter) {
            let value = value.clone();
            unsafe {
                self.buf.write(index, value);
            }
            self.len += 1;
        }
        iter.as_slice()
    }

    pub fn resize(&mut self, len: usize, value: T) -> Option<()>
    where
        T: Clone,
    {
        if len > N {
            return None;
        }
        if len <= self.len {
            self.truncate(len);
            return Some(());
        }
        let last = len - 1;
        for index in self.len..last {
            let value = value.clone();
            unsafe {
                self.buf.write(index, value);
            }
            self.len += 1;
        }
        unsafe {
            self.buf.write(last, value);
        }
        self.len += 1;
        Some(())
    }

    pub fn resize_with<F>(&mut self, len: usize, mut f: F) -> Option<()>
    where
        F: FnMut(usize) -> T,
    {
        if len > N {
            return None;
        }
        if len <= self.len {
            self.truncate(len);
            return Some(());
        }
        for index in self.len..len {
            let value = f(index);
            unsafe {
                self.buf.write(index, value);
            }
            self.len += 1;
        }
        Some(())
    }

    pub fn truncate(&mut self, len: usize) {
        if len >= self.len {
            return;
        }
        let to_drop = len..self.len;
        self.len = len;
        unsafe {
            self.buf.assume_init_drop(to_drop);
        }
    }

    pub const fn split_off(&mut self, at: usize) -> Option<Self> {
        if at > self.len {
            return None;
        }
        let mut result = Self::new();
        let dst_len = self.len - at;
        self.len = at;
        unsafe {
            let src_index = at;
            let dst_index = 0;
            let count = dst_len;
            buf::copy_nonoverlapping(&self.buf, &mut result.buf, src_index, dst_index, count);
        }
        result.len = dst_len;
        Some(result)
    }

    pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { self.buf.get_unchecked_mut(self.len..N) }
    }

    pub fn clear(&mut self) {
        let to_drop = 0..self.len;
        self.len = 0;
        unsafe {
            self.buf.assume_init_drop(to_drop);
        }
    }
}

impl<T, const N: usize> fmt::Debug for InlineVec<T, N>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T, const N: usize> Default for InlineVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Clone for InlineVec<T, N>
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

impl<T, const N: usize> Deref for InlineVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for InlineVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> AsRef<[T]> for InlineVec<T, N> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> AsMut<[T]> for InlineVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Borrow<[T]> for InlineVec<T, N> {
    fn borrow(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> BorrowMut<[T]> for InlineVec<T, N> {
    fn borrow_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> Hash for InlineVec<T, N>
where
    T: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_slice().hash(state);
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineVec<U, M>> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineDeque<U, M>> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self.iter().eq(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<[U; M]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; M]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T, const N: usize, U> PartialEq<[U]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineVec<U, M>> for [T; N]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, U, const M: usize> PartialEq<InlineVec<U, M>> for [T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize> Eq for InlineVec<T, N> where T: Eq {}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineVec<U, M>> for InlineVec<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineVec<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineDeque<U, M>> for InlineVec<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineDeque<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<[U; M]> for InlineVec<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &[U; M]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U> PartialOrd<[U]> for InlineVec<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &[U]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineVec<U, M>> for [T; N]
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineVec<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, U, const M: usize> PartialOrd<InlineVec<U, M>> for [T]
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineVec<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> Ord for InlineVec<T, N>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T, const N: usize, I> Index<I> for InlineVec<T, N>
where
    I: SliceIndex<[T]>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_slice().index(index)
    }
}

impl<T, const N: usize, I> IndexMut<I> for InlineVec<T, N>
where
    I: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_slice().index_mut(index)
    }
}

impl<T, const N: usize> Drop for InlineVec<T, N> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const N: usize> InlineVec<T, N> {
    pub(crate) const fn buf(&self) -> &Buf<T, N> {
        &self.buf
    }

    pub(crate) const unsafe fn buf_mut(&mut self) -> &mut Buf<T, N> {
        &mut self.buf
    }
}
