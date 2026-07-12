use super::InlineString;
use crate::buf;
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
        let len = value.len();
        if len > N {
            return Err(StringError::capacity_overflow().with_value(value));
        }
        let mut result = Self::new();
        unsafe {
            let src_index = 0;
            let dst_index = 0;
            let count = len;
            buf::copy_nonoverlapping(
                value.buf(),
                result.vec.buf_mut(),
                src_index,
                dst_index,
                count,
            );
            result.vec.set_len(len);
        }
        Ok(result)
    }
}

impl<const N: usize, const M: usize> TryFrom<InlineDeque<u8, M>> for InlineString<N> {
    type Error = StringError<InlineDeque<u8, M>>;

    fn try_from(value: InlineDeque<u8, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            return Err(StringError::capacity_overflow().with_value(value));
        }
        let (prefix, suffix) = value.slice_spans();
        let mut result = Self::new();
        unsafe {
            let src_index = prefix.start;
            let dst_index = 0;
            let count = prefix.len;
            buf::copy_nonoverlapping(
                value.buf(),
                result.vec.buf_mut(),
                src_index,
                dst_index,
                count,
            );
            let src_index = suffix.start;
            let dst_index = prefix.len;
            let count = suffix.len;
            buf::copy_nonoverlapping(
                value.buf(),
                result.vec.buf_mut(),
                src_index,
                dst_index,
                count,
            );
            result.vec.set_len(len);
        }
        Ok(result)
    }
}
