use super::InlineDeque;
use crate::buf;
use crate::error::Error;
use crate::string::InlineString;
use crate::vec::InlineVec;
use core::mem::ManuallyDrop;
use core::ptr;

impl<T, const N: usize, const M: usize> TryFrom<[T; M]> for InlineDeque<T, N> {
    type Error = Error<[T; M]>;

    fn try_from(value: [T; M]) -> Result<Self, Self::Error> {
        if M > N {
            let error = Error::capacity_overflow(value);
            return Err(error);
        }
        let value = ManuallyDrop::new(value);
        let mut result = Self::new();
        unsafe {
            let src = value.as_ptr();
            let dst = result.buf.as_mut_ptr();
            let count = M;
            ptr::copy_nonoverlapping(src, dst, count);
        }
        result.len = M;
        Ok(result)
    }
}

impl<T, const N: usize, const M: usize> TryFrom<InlineVec<T, M>> for InlineDeque<T, N> {
    type Error = Error<InlineVec<T, M>>;

    fn try_from(value: InlineVec<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            let error = Error::capacity_overflow(value);
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
            let error = Error::capacity_overflow(value);
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
