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
        self.iter().eq(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<[U; M]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U; M]) -> bool {
        self.iter().eq(other)
    }
}

impl<T, const N: usize, U> PartialEq<[U]> for InlineDeque<T, N>
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &[U]) -> bool {
        self.iter().eq(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialEq<InlineDeque<U, M>> for [T; N]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self.iter().eq(other)
    }
}

impl<T, U, const M: usize> PartialEq<InlineDeque<U, M>> for [T]
where
    T: PartialEq<U>,
{
    fn eq(&self, other: &InlineDeque<U, M>) -> bool {
        self.iter().eq(other)
    }
}

impl<T, const N: usize> Eq for InlineDeque<T, N> where T: Eq {}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineDeque<U, M>> for InlineDeque<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineDeque<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineVec<U, M>> for InlineDeque<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineVec<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<[U; M]> for InlineDeque<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &[U; M]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U> PartialOrd<[U]> for InlineDeque<T, N>
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &[U]) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, const N: usize, U, const M: usize> PartialOrd<InlineDeque<U, M>> for [T; N]
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineDeque<U, M>) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T, U, const M: usize> PartialOrd<InlineDeque<U, M>> for [T]
where
    T: PartialOrd<U>,
{
    fn partial_cmp(&self, other: &InlineDeque<U, M>) -> Option<Ordering> {
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
