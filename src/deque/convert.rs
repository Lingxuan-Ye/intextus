use super::InlineDeque;
use crate::vec::InlineVec;
use core::mem::ManuallyDrop;
use core::ptr;

impl<T, const N: usize, const M: usize> TryFrom<InlineVec<T, M>> for InlineDeque<T, N> {
    type Error = InlineVec<T, M>;

    fn try_from(value: InlineVec<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            return Err(value);
        }
        let value = ManuallyDrop::new(value);
        let mut result = Self::new();
        let src = value.as_ptr();
        let dst = result.buf.as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src, dst, len);
        }
        result.len = len;
        Ok(result)
    }
}
