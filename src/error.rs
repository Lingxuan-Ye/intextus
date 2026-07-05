use core::char::DecodeUtf16Error;
use core::error;
use core::fmt;
use core::num::NonZero;
use core::str::Utf8Error;

#[derive(PartialEq, Eq)]
pub struct Error<T = ()> {
    kind: ErrorKind,
    value: T,
}

impl<T> Error<T> {
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub(crate) const fn index_out_of_bounds(index: usize, upper: UpperBound, value: T) -> Self {
        let inner = IndexOutOfBounds::new(index, upper);
        let kind = ErrorKind::IndexOutOfBounds(inner);
        Self { kind, value }
    }

    pub(crate) const fn full<const N: usize>(value: T) -> Self {
        let inner = CapacityOverflow::full::<N>();
        let kind = ErrorKind::CapacityOverflow(inner);
        Self { kind, value }
    }

    /// The caller must ensure that:
    ///
    /// - `len != Some(0)`.
    pub(crate) const unsafe fn capacity_overflow<const N: usize>(
        len: Option<usize>,
        value: T,
    ) -> Self {
        let inner = unsafe { CapacityOverflow::new_unchecked::<N>(len) };
        let kind = ErrorKind::CapacityOverflow(inner);
        Self { kind, value }
    }
}

impl<T> fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::IndexOutOfBounds(inner) => write!(f, "Error::IndexOutOfBounds({inner:?})"),
            ErrorKind::CapacityOverflow(inner) => write!(f, "Error::CapacityOverflow({inner:?})"),
        }
    }
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::IndexOutOfBounds(inner) => write!(f, "{inner}"),
            ErrorKind::CapacityOverflow(inner) => write!(f, "{inner}"),
        }
    }
}

impl<T> error::Error for Error<T> {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    IndexOutOfBounds(IndexOutOfBounds),
    CapacityOverflow(CapacityOverflow),
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IndexOutOfBounds(_) => write!(f, "ErrorKind::IndexOutOfBounds"),
            Self::CapacityOverflow(_) => write!(f, "ErrorKind::CapacityOverflow"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringError {
    Utf8Error(Utf8Error),
    Utf16Error(DecodeUtf16Error),
    NotCharBoundary(usize),
    IndexOutOfBounds(IndexOutOfBounds),
    CapacityOverflow(CapacityOverflow),
}

impl StringError {
    pub(crate) const fn index_out_of_bounds(index: usize, upper: UpperBound) -> Self {
        let inner = IndexOutOfBounds::new(index, upper);
        Self::IndexOutOfBounds(inner)
    }

    /// The caller must ensure that:
    ///
    /// - `len != Some(0)`.
    pub(crate) const unsafe fn capacity_overflow<const N: usize>(len: Option<usize>) -> Self {
        let inner = unsafe { CapacityOverflow::new_unchecked::<N>(len) };
        Self::CapacityOverflow(inner)
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Error(inner) => write!(f, "{inner}"),
            Self::Utf16Error(inner) => write!(f, "{inner}"),
            Self::NotCharBoundary(index) => write!(f, "index {index} is not a char boundary"),
            Self::IndexOutOfBounds(inner) => write!(f, "{inner}"),
            Self::CapacityOverflow(inner) => write!(f, "{inner}"),
        }
    }
}

impl error::Error for StringError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexOutOfBounds {
    index: usize,
    upper: UpperBound,
}

impl IndexOutOfBounds {
    pub(crate) const fn new(index: usize, upper: UpperBound) -> Self {
        Self { index, upper }
    }
}

impl fmt::Display for IndexOutOfBounds {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.upper {
            UpperBound::Included(upper) => {
                write!(
                    f,
                    "index out of bounds: index {index} is not in 0..={upper}",
                    index = self.index,
                )
            }
            UpperBound::Excluded(upper) => {
                write!(
                    f,
                    "index out of bounds: index {index} is not in 0..{upper}",
                    index = self.index,
                )
            }
        }
    }
}

impl error::Error for IndexOutOfBounds {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpperBound {
    Included(usize),
    Excluded(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityOverflow {
    len: Option<NonZero<usize>>,
    capacity: usize,
}

impl CapacityOverflow {
    pub(crate) const fn full<const N: usize>() -> Self {
        let len = N.checked_add(1);
        unsafe { Self::new_unchecked::<N>(len) }
    }

    /// The caller must ensure that:
    ///
    /// - `len != Some(0)`.
    pub(crate) const unsafe fn new_unchecked<const N: usize>(len: Option<usize>) -> Self {
        let len = match len {
            None => None,
            Some(len) => unsafe { Some(NonZero::new_unchecked(len)) },
        };
        let capacity = N;
        Self { len, capacity }
    }
}

impl fmt::Display for CapacityOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.len {
            None => {
                write!(
                    f,
                    "capacity overflow: the resulting len would be greater than usize::MAX",
                )
            }
            Some(len) => {
                write!(
                    f,
                    "capacity overflow: the resulting len would be {len}, but the capacity is {capacity}",
                    len = len.get(),
                    capacity = self.capacity,
                )
            }
        }
    }
}

impl error::Error for CapacityOverflow {}
