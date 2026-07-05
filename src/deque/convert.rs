use super::InlineDeque;
use crate::buf;
use crate::error::Error;
use crate::string::InlineString;
use crate::vec::InlineVec;
use core::mem::ManuallyDrop;

impl<T, const N: usize, const M: usize> TryFrom<InlineVec<T, M>> for InlineDeque<T, N> {
    type Error = Error<InlineVec<T, M>>;

    fn try_from(value: InlineVec<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            let error = unsafe { Error::capacity_overflow::<N>(Some(len), value) };
            return Err(error);
        }
        let value = ManuallyDrop::new(value);
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

impl<const N: usize, const M: usize> TryFrom<InlineString<M>> for InlineDeque<u8, N> {
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
