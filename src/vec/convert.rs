use super::InlineVec;
use crate::buf;
use crate::deque::InlineDeque;
use crate::error::Error;
use crate::string::InlineString;
use core::mem::ManuallyDrop;

impl<T, const N: usize, const M: usize> TryFrom<InlineDeque<T, M>> for InlineVec<T, N> {
    type Error = Error<InlineDeque<T, M>>;

    fn try_from(value: InlineDeque<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            let error = unsafe { Error::capacity_overflow::<N>(Some(len), value) };
            return Err(error);
        }
        let value = ManuallyDrop::new(value);
        let (prefix, suffix) = value.slice_spans();
        let mut result = Self::new();
        unsafe {
            let src_index = prefix.start;
            let dst_index = 0;
            let count = prefix.len;
            buf::copy_nonoverlapping(value.buf(), &mut result.buf, src_index, dst_index, count);
            let src_index = suffix.start;
            let dst_index = prefix.len;
            let count = suffix.len;
            buf::copy_nonoverlapping(value.buf(), &mut result.buf, src_index, dst_index, count);
        }
        result.len = len;
        Ok(result)
    }
}

impl<const N: usize, const M: usize> TryFrom<InlineString<M>> for InlineVec<u8, N> {
    type Error = Error<InlineString<M>>;

    fn try_from(value: InlineString<M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            let error = unsafe { Error::capacity_overflow::<N>(Some(len), value) };
            return Err(error);
        }
        let mut result = Self::new();
        unsafe {
            let src_index = 0;
            let dst_index = 0;
            let count = len;
            buf::copy_nonoverlapping(value.buf(), &mut result.buf, src_index, dst_index, count);
        }
        result.len = len;
        Ok(result)
    }
}
