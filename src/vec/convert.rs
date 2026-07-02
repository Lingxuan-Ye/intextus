use super::InlineVec;
use crate::deque::InlineDeque;
use core::mem::ManuallyDrop;
use core::ptr;

impl<T, const N: usize, const M: usize> TryFrom<InlineDeque<T, M>> for InlineVec<T, N> {
    type Error = InlineDeque<T, M>;

    fn try_from(value: InlineDeque<T, M>) -> Result<Self, Self::Error> {
        let len = value.len();
        if len > N {
            return Err(value);
        }
        let value = ManuallyDrop::new(value);
        let (prefix, suffix) = value.as_slices();
        let mut result = Self::new();
        let dst_base = result.as_mut_ptr();
        unsafe {
            let dst = dst_base;
            ptr::copy_nonoverlapping(prefix.as_ptr(), dst, prefix.len());
            let dst = dst_base.add(prefix.len());
            ptr::copy_nonoverlapping(suffix.as_ptr(), dst, suffix.len());
        }
        result.len = len;
        Ok(result)
    }
}
