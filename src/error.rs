use core::char::DecodeUtf16Error;
use core::error;
use core::fmt;
use core::str::Utf8Error;

#[derive(PartialEq, Eq)]
pub struct Error<T = ()> {
    kind: ErrorKind,
    value: T,
}

impl<T> Error<T> {
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn into_inner(self) -> T {
        self.value
    }

    pub(crate) const fn capacity_overflow(value: T) -> Self {
        let inner = CapacityOverflow::new();
        let kind = ErrorKind::CapacityOverflow(inner);
        Self { kind, value }
    }

    pub(crate) const fn index_out_of_bounds(index: usize, upper: UpperBound, value: T) -> Self {
        let inner = IndexOutOfBounds::new(index, upper);
        let kind = ErrorKind::IndexOutOfBounds(inner);
        Self { kind, value }
    }
}

impl<T> fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::CapacityOverflow(inner) => write!(f, "Error::CapacityOverflow({inner:?})"),
            ErrorKind::IndexOutOfBounds(inner) => write!(f, "Error::IndexOutOfBounds({inner:?})"),
        }
    }
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::CapacityOverflow(inner) => write!(f, "{inner}"),
            ErrorKind::IndexOutOfBounds(inner) => write!(f, "{inner}"),
        }
    }
}

impl<T> error::Error for Error<T> {}

#[derive(PartialEq, Eq)]
pub enum ErrorKind {
    CapacityOverflow(CapacityOverflow),
    IndexOutOfBounds(IndexOutOfBounds),
}

impl fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CapacityOverflow(_) => write!(f, "ErrorKind::CapacityOverflow"),
            Self::IndexOutOfBounds(_) => write!(f, "ErrorKind::IndexOutOfBounds"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StringError {
    Utf8Error(Utf8Error),
    Utf16Error(DecodeUtf16Error),
    CapacityOverflow(CapacityOverflow),
    IndexOutOfBounds(IndexOutOfBounds),
    NotCharBoundary(NotCharBoundary),
}

impl StringError {
    pub(crate) const fn capacity_overflow() -> Self {
        let inner = CapacityOverflow::new();
        Self::CapacityOverflow(inner)
    }

    pub(crate) const fn index_out_of_bounds(index: usize, upper: UpperBound) -> Self {
        let inner = IndexOutOfBounds::new(index, upper);
        Self::IndexOutOfBounds(inner)
    }

    pub(crate) const fn not_char_boundary(index: usize) -> Self {
        let inner = NotCharBoundary::new(index);
        Self::NotCharBoundary(inner)
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Error(inner) => write!(f, "{inner}"),
            Self::Utf16Error(inner) => write!(f, "{inner}"),
            Self::CapacityOverflow(inner) => write!(f, "{inner}"),
            Self::IndexOutOfBounds(inner) => write!(f, "{inner}"),
            Self::NotCharBoundary(inner) => write!(f, "{inner}"),
        }
    }
}

impl error::Error for StringError {}

#[derive(Debug, PartialEq, Eq)]
pub struct CapacityOverflow(());

impl CapacityOverflow {
    pub(crate) const fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for CapacityOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "capacity overflow")
    }
}

impl error::Error for CapacityOverflow {}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum UpperBound {
    Included(usize),
    Excluded(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotCharBoundary {
    index: usize,
}

impl NotCharBoundary {
    pub(crate) const fn new(index: usize) -> Self {
        Self { index }
    }
}

impl fmt::Display for NotCharBoundary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "index {index} is not a char boundary",
            index = self.index,
        )
    }
}

impl error::Error for NotCharBoundary {}
