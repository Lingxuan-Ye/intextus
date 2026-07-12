use super::InlineString;
use crate::deque::InlineDeque;
use crate::error::StringError;
use crate::vec::InlineVec;

impl<const N: usize> TryFrom<&str> for InlineString<N> {
    type Error = StringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        result.push_str(value)?;
        Ok(result)
    }
}

impl<const N: usize, const M: usize> TryFrom<InlineVec<u8, M>> for InlineString<N> {
    type Error = StringError<InlineVec<u8, M>>;

    fn try_from(value: InlineVec<u8, M>) -> Result<Self, Self::Error> {
        let bytes = value.as_slice();
        Self::from_utf8(bytes).map_err(|error| error.with_value(value))
    }
}

impl<const N: usize, const M: usize> TryFrom<InlineDeque<u8, M>> for InlineString<N> {
    type Error = StringError<InlineDeque<u8, M>>;

    fn try_from(value: InlineDeque<u8, M>) -> Result<Self, Self::Error> {
        let mut result = Self::new();
        let (prefix, suffix) = value.as_slices();
        if let Err(error) = result.push_utf8(prefix) {
            return Err(error.with_value(value));
        }
        if let Err(error) = result.push_utf8(suffix) {
            return Err(error.with_value(value));
        }
        Ok(result)
    }
}
