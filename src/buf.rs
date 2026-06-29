use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::slice::SliceIndex;

#[derive(Debug)]
pub(crate) struct Buf<T, const N: usize>([MaybeUninit<T>; N]);

impl<T, const N: usize> Buf<T, N> {
    pub(crate) const fn new() -> Self {
        Self([const { MaybeUninit::uninit() }; N])
    }

    pub(crate) const fn as_ptr(&self) -> *const MaybeUninit<T> {
        self.0.as_ptr()
    }

    pub(crate) const fn as_mut_ptr(&mut self) -> *mut MaybeUninit<T> {
        self.0.as_mut_ptr()
    }

    pub(crate) unsafe fn write(&mut self, index: usize, value: T) -> &mut T {
        unsafe { self.get_unchecked_mut(index).write(value) }
    }

    pub(crate) unsafe fn assume_init_ref(&self, index: usize) -> &T {
        unsafe { self.get_unchecked(index).assume_init_ref() }
    }

    pub(crate) unsafe fn assume_init_mut(&mut self, index: usize) -> &mut T {
        unsafe { self.get_unchecked_mut(index).assume_init_mut() }
    }

    pub(crate) unsafe fn assume_init_read(&self, index: usize) -> T {
        unsafe { self.get_unchecked(index).assume_init_read() }
    }

    pub(crate) unsafe fn copy_within(&mut self, src: usize, dst: usize, count: usize) {
        let base = self.as_mut_ptr();
        unsafe {
            let src = base.add(src);
            let dst = base.add(dst);
            ptr::copy(src, dst, count);
        }
    }

    pub(crate) unsafe fn assume_init_drop<I>(&mut self, index: I)
    where
        I: SliceIndex<[MaybeUninit<T>], Output = [MaybeUninit<T>]>,
    {
        unsafe {
            self.get_unchecked_mut(index).assume_init_drop();
        }
    }
}

impl<T, const N: usize> Deref for Buf<T, N> {
    type Target = [MaybeUninit<T>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for Buf<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
