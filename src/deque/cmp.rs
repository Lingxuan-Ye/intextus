use super::InlineDeque;
use crate::vec::InlineVec;
use core::cmp::Ordering;

impl<T, const N: usize, U, const M: usize> PartialEq<InlineDeque<U, M>> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self.iter().eq(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineVec<U, M>> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<[U; M]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; M]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<&[U; M]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U; M]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<&mut [U; M]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&mut [U; M]) -> bool {
        self.eq(other.as_slice())
    }
}

impl<T, const N: usize, U> PartialEq<[U]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let lhs = self.as_slices();
        let mid = lhs.0.len();
        let rhs = unsafe { other.split_at_unchecked(mid) };
        lhs.0.eq(rhs.0) && lhs.1.eq(rhs.1)
    }
}

impl<T, const N: usize, U> PartialEq<&[U]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U]) -> bool {
        self.eq(*other)
    }
}

impl<T, const N: usize, U> PartialEq<&mut [U]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&mut [U]) -> bool {
        self.eq(*other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineDeque<U, M>> for [T; N]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self.as_slice().eq(other)
    }
}

impl<T, U, const M: usize> PartialEq<InlineDeque<U, M>> for [T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        let rhs = other.as_slices();
        let mid = rhs.0.len();
        let lhs = unsafe { self.split_at_unchecked(mid) };
        lhs.0.eq(rhs.0) && lhs.1.eq(rhs.1)
    }
}

impl<T, U, const M: usize> PartialEq<InlineDeque<U, M>> for &[T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self[..].eq(other)
    }
}

impl<T, U, const M: usize> PartialEq<InlineDeque<U, M>> for &mut [T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self[..].eq(other)
    }
}

impl<T, const N: usize> Eq for InlineDeque<T, N> where T: Eq {}

impl<T, const N: usize, const M: usize> PartialOrd<InlineDeque<T, M>> for InlineDeque<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &InlineDeque<T, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, const M: usize> PartialOrd<InlineVec<T, M>> for InlineDeque<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &InlineVec<T, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, const M: usize> PartialOrd<[T; M]> for InlineDeque<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &[T; M]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> PartialOrd<[T]> for InlineDeque<T, N>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &[T]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, const M: usize> PartialOrd<InlineDeque<T, M>> for [T; N]
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &InlineDeque<T, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> PartialOrd<InlineDeque<T, N>> for [T]
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &InlineDeque<T, N>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize> Ord for InlineDeque<T, N>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}
