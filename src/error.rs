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

    pub fn into_value(self) -> T {
        self.value
    }
}

impl Error {
    pub(crate) const fn capacity_overflow() -> Self {
        let error = CapacityOverflow::new();
        let kind = ErrorKind::CapacityOverflow(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn index_out_of_bounds(index: usize, upper: UpperBound) -> Self {
        let error = IndexOutOfBounds::new(index, upper);
        let kind = ErrorKind::IndexOutOfBounds(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn with_value<T>(self, value: T) -> Error<T> {
        let kind = self.kind;
        Error { kind, value }
    }
}

impl<T> fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::CapacityOverflow(error) => write!(f, "{error}"),
            ErrorKind::IndexOutOfBounds(error) => write!(f, "{error}"),
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
            Self::CapacityOverflow(error) => write!(f, "{error:?}"),
            Self::IndexOutOfBounds(error) => write!(f, "{error:?}"),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct StringError<T = ()> {
    kind: StringErrorKind,
    value: T,
}

impl<T> StringError<T> {
    pub const fn kind(&self) -> &StringErrorKind {
        &self.kind
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

impl StringError {
    pub(crate) const fn utf8_error(error: Utf8Error) -> Self {
        let kind = StringErrorKind::Utf8Error(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn utf16_error(error: DecodeUtf16Error) -> Self {
        let kind = StringErrorKind::Utf16Error(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn capacity_overflow() -> Self {
        let error = CapacityOverflow::new();
        let kind = StringErrorKind::CapacityOverflow(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn not_char_boundary(index: usize) -> Self {
        let error = NotCharBoundary::new(index);
        let kind = StringErrorKind::NotCharBoundary(error);
        let value = ();
        Self { kind, value }
    }

    pub(crate) const fn with_value<T>(self, value: T) -> StringError<T> {
        let kind = self.kind;
        StringError { kind, value }
    }
}

impl<T> fmt::Debug for StringError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StringError")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Display for StringError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            StringErrorKind::Utf8Error(error) => write!(f, "{error}"),
            StringErrorKind::Utf16Error(error) => write!(f, "{error}"),
            StringErrorKind::CapacityOverflow(error) => write!(f, "{error}"),
            StringErrorKind::NotCharBoundary(error) => write!(f, "{error}"),
        }
    }
}

impl<T> error::Error for StringError<T> {}

#[derive(PartialEq, Eq)]
pub enum StringErrorKind {
    Utf8Error(Utf8Error),
    Utf16Error(DecodeUtf16Error),
    CapacityOverflow(CapacityOverflow),
    NotCharBoundary(NotCharBoundary),
}

impl fmt::Debug for StringErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Utf8Error(error) => write!(f, "{error:?}"),
            Self::Utf16Error(error) => write!(f, "{error:?}"),
            Self::CapacityOverflow(error) => write!(f, "{error:?}"),
            Self::NotCharBoundary(error) => write!(f, "{error:?}"),
        }
    }
}

#[derive(PartialEq, Eq)]
pub struct CapacityOverflow(());

impl CapacityOverflow {
    pub(crate) const fn new() -> Self {
        Self(())
    }
}

impl fmt::Debug for CapacityOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CapacityOverflow").finish_non_exhaustive()
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
