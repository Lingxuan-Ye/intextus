use super::InlineVec;
use crate::deque::InlineDeque;
use core::cmp::Ordering;

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

impl<T, const N: usize, U, const M: usize> PartialEq<&[U; M]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U; M]) -> bool {
        self.as_slice().eq(other.as_slice())
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<&mut [U; M]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&mut [U; M]) -> bool {
        self.as_slice().eq(other.as_slice())
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

impl<T, const N: usize, U> PartialEq<&[U]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&[U]) -> bool {
        self.as_slice().eq(*other)
    }
}

impl<T, const N: usize, U> PartialEq<&mut [U]> for InlineVec<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &&mut [U]) -> bool {
        self.as_slice().eq(*other)
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

impl<T, U, const M: usize> PartialEq<InlineVec<U, M>> for &[T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self[..].eq(other.as_slice())
    }
}

impl<T, U, const M: usize> PartialEq<InlineVec<U, M>> for &mut [T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineVec<U, M>) -> bool {
        self[..].eq(other.as_slice())
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
