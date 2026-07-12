use super::InlineVec;
use crate::buf;
use crate::deque::InlineDeque;
use crate::error::Error;
use crate::string::InlineString;
use core::mem::ManuallyDrop;
use core::ptr;

impl<T, const N: usize, const M: usize> TryFrom<[T; M]> for InlineVec<T, N> {
    type Error = Error<[T; M]>;

    fn try_from(value: [T; M]) -> Result<Self, Self::Error> {
        if M > N {
            return Err(Error::capacity_overflow().with_value(value));
        }
        let value = ManuallyDrop::new(value);
        let mut result = Self::new();
        unsafe {
            let src = value.as_ptr();
            let dst = result.as_mut_ptr();
            let count = M;
            ptr::copy_nonoverlapping(src, dst, count);
        }
        result.len = M;
        Ok(result)
    }
}

impl<T, const N: usize, const M: usize> TryFrom<InlineDeque<T, M>> for InlineVec<T, N> {
    type Error = Error<InlineDeque<T, M>>;

    fn try_from(value: InlineDeque<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            return Err(Error::capacity_overflow().with_value(value));
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
            return Err(Error::capacity_overflow().with_value(value));
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
