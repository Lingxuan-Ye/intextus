use super::InlineString;
use core::cmp::Ordering;

impl<const N: usize, const M: usize> PartialEq<InlineString<M>> for InlineString<N> {
    fn eq(&self, other: &InlineString<M>) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<const N: usize> PartialEq<str> for InlineString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl<const N: usize> PartialEq<&str> for InlineString<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<const N: usize> PartialEq<&mut str> for InlineString<N> {
    fn eq(&self, other: &&mut str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<const N: usize> PartialEq<InlineString<N>> for str {
    fn eq(&self, other: &InlineString<N>) -> bool {
        self.eq(other.as_str())
    }
}

impl<const N: usize> PartialEq<InlineString<N>> for &str {
    fn eq(&self, other: &InlineString<N>) -> bool {
        self[..].eq(other.as_str())
    }
}

impl<const N: usize> PartialEq<InlineString<N>> for &mut str {
    fn eq(&self, other: &InlineString<N>) -> bool {
        self[..].eq(other.as_str())
    }
}

impl<const N: usize> Eq for InlineString<N> {}

impl<const N: usize, const M: usize> PartialOrd<InlineString<M>> for InlineString<N> {
    fn partial_cmp(&self, other: &InlineString<M>) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<const N: usize> Ord for InlineString<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}
