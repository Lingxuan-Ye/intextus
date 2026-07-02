use core::hint;
use core::mem;
use core::ops::Range as LegacyRange;
use core::ptr;
use core::range::Range;

#[derive(Debug)]
pub(crate) struct Buf<T, const N: usize>([mem::MaybeUninit<T>; N]);

impl<T, const N: usize> Buf<T, N> {
    pub(crate) const fn new() -> Self {
        Self([const { mem::MaybeUninit::uninit() }; N])
    }

    pub(crate) const fn as_ptr(&self) -> *const T {
        self.0.as_ptr().cast()
    }

    pub(crate) const fn as_mut_ptr(&mut self) -> *mut T {
        self.0.as_mut_ptr().cast()
    }

    /// The caller must ensure that:
    ///
    /// - `i < N`.
    /// - `j < N`.
    pub(crate) const unsafe fn swap(&mut self, i: usize, j: usize) {
        let base = self.as_mut_ptr();
        unsafe {
            let x = base.add(i);
            let y = base.add(j);
            ptr::swap(x, y);
        }
    }

    /// The caller must ensure that:
    ///
    /// - `src_index + count <= N`.
    /// - `dst_index + count <= N`.
    pub(crate) const unsafe fn copy_within(
        &mut self,
        src_index: usize,
        dst_index: usize,
        count: usize,
    ) {
        let base = self.as_mut_ptr();
        unsafe {
            let src = base.add(src_index);
            let dst = base.add(dst_index);
            ptr::copy(src, dst, count);
        }
    }

    /// The caller must ensure that:
    ///
    /// - `src_index + count <= N`.
    /// - `dst_index + count <= N`.
    /// - `src_index..(src_index + count)` and `dst_index..(dst_index + count)` must
    ///   not overlap.
    pub(crate) const unsafe fn copy_within_nonoverlapping(
        &mut self,
        src_index: usize,
        dst_index: usize,
        count: usize,
    ) {
        let base = self.as_mut_ptr();
        unsafe {
            let src = base.add(src_index);
            let dst = base.add(dst_index);
            ptr::copy_nonoverlapping(src, dst, count);
        }
    }

    pub(crate) unsafe fn get_unchecked<I>(&self, index: I) -> &I::Output
    where
        I: BufIndex<T, N>,
    {
        unsafe { index.get_unchecked(self) }
    }

    pub(crate) unsafe fn get_unchecked_mut<I>(&mut self, index: I) -> &mut I::Output
    where
        I: BufIndex<T, N>,
    {
        unsafe { index.get_unchecked_mut(self) }
    }

    /// The caller must ensure that:
    ///
    /// - `index` is in bounds.
    pub(crate) unsafe fn write<'a, I>(
        &'a mut self,
        index: I,
        value: <I::Output as MaybeUninit>::Init,
    ) -> &'a mut <I::Output as MaybeUninit>::Init
    where
        I: BufIndex<T, N>,
        I::Output: MaybeUninitExt + 'a,
    {
        unsafe { self.get_unchecked_mut(index).write(value) }
    }

    /// The caller must ensure that:
    ///
    /// - `index` is in bounds.
    /// - The value at `index` is initialized and valid.
    pub(crate) unsafe fn assume_init_read<'a, I>(
        &'a self,
        index: I,
    ) -> <I::Output as MaybeUninit>::Init
    where
        I: BufIndex<T, N>,
        I::Output: MaybeUninitExt + 'a,
    {
        unsafe { self.get_unchecked(index).assume_init_read() }
    }

    /// The caller must ensure that:
    ///
    /// - `index` is in bounds.
    /// - The value at `index` is initialized and valid.
    pub(crate) unsafe fn assume_init_ref<'a, I>(
        &'a self,
        index: I,
    ) -> &'a <I::Output as MaybeUninit>::Init
    where
        I: BufIndex<T, N>,
        I::Output: 'a,
    {
        unsafe { self.get_unchecked(index).assume_init_ref() }
    }

    /// The caller must ensure that:
    ///
    /// - `index` is in bounds.
    /// - The value at `index` is initialized and valid.
    pub(crate) unsafe fn assume_init_mut<'a, I>(
        &'a mut self,
        index: I,
    ) -> &'a mut <I::Output as MaybeUninit>::Init
    where
        I: BufIndex<T, N>,
        I::Output: 'a,
    {
        unsafe { self.get_unchecked_mut(index).assume_init_mut() }
    }

    /// The caller must ensure that:
    ///
    /// - `index` is in bounds.
    /// - The slice at `index` is initialized and valid.
    pub(crate) unsafe fn assume_init_drop<'a, I>(&'a mut self, index: I)
    where
        I: BufIndex<T, N>,
        I::Output: 'a,
    {
        unsafe {
            self.get_unchecked_mut(index).assume_init_drop();
        }
    }
}

impl<T, const N: usize> Buf<T, N> {
    /// The caller must ensure that:
    ///
    /// - `src_index < N`.
    /// - `dst_index < N`.
    /// - `count <= N`.
    pub(crate) const unsafe fn wrap_copy_within(
        &mut self,
        src_index: usize,
        dst_index: usize,
        count: usize,
    ) {
        if size_of::<T>() == 0 || src_index == dst_index || count == 0 {
            return;
        }

        let src_to_dst = Self::wrap_sub(dst_index, src_index);
        let dst_to_src = N - src_to_dst;

        // In the diagrams below, `_` denotes an irrelevant value and does not imply
        // whether the slot is initialized.

        if count == N {
            //    . . . S . . . . . .
            // 0 [A B C D E F G H I J] old
            // 1 [_ _ _ A B C D E F G] new
            // 2 [H I J A B C D E F G] new
            //    . . . . . . D . . .
            let mut buf = Self::new();
            unsafe {
                copy_nonoverlapping(self, &mut buf, 0, src_to_dst, dst_to_src);
                copy_nonoverlapping(self, &mut buf, dst_to_src, 0, src_to_dst);
            }
            *self = buf;
            return;
        }

        let src_to_end = N - src_index;
        let dst_to_end = N - dst_index;
        let src_wraps = count > src_to_end;
        let dst_wraps = count > dst_to_end;

        match (src_index < dst_index, src_wraps, dst_wraps) {
            (true, false, false) => unsafe {
                match count <= src_to_dst {
                    true => {
                        //      S . . .
                        // 0 [_ A B C D _ _ _ _ _]
                        // 1 [_ A B C D A B C D _]
                        //              D . . .
                        self.copy_within_nonoverlapping(src_index, dst_index, count);
                    }
                    false => {
                        //      S . . .
                        // 0 [_ A B C D _ _ _ _ _]
                        // 1 [_ A B C A B C D _ _]
                        //            D . . .
                        self.copy_within(src_index, dst_index, count);
                    }
                }
            },
            (true, false, true) => unsafe {
                match (count <= src_to_dst, count <= dst_to_src) {
                    (true, true) => {
                        //      S . . .
                        // 0 [_ A B C D _ _ _ _ _]
                        // 1 [_ A B C D _ _ A B C]
                        // 2 [D A B C D _ _ A B C]
                        //    .             D . .
                        self.copy_within_nonoverlapping(src_index, dst_index, dst_to_end);
                        self.copy_within_nonoverlapping(
                            src_index + dst_to_end,
                            0,
                            count - dst_to_end,
                        );
                    }
                    (true, false) => {
                        //      S . . .
                        // 0 [_ A B C D _ _ _ _ _]
                        // 1 [_ A B C D _ _ _ _ A]
                        // 2 [B C D C D _ _ _ _ A]
                        //    . . .             D
                        self.copy_within_nonoverlapping(src_index, dst_index, dst_to_end);
                        self.copy_within(src_index + dst_to_end, 0, count - dst_to_end);
                    }
                    (false, true) => {
                        //              S . . .
                        // 0 [_ _ _ _ _ A B C D _]
                        // 1 [D _ _ _ _ A B C D _]
                        // 2 [D _ _ _ _ A B A B C]
                        //    .             D . .
                        self.copy_within_nonoverlapping(
                            src_index + dst_to_end,
                            0,
                            count - dst_to_end,
                        );
                        self.copy_within(src_index, dst_index, dst_to_end);
                    }
                    (false, false) => {
                        //      S . . . . . . .
                        // 0 [_ A B C D E F G H _] old
                        // 1 [_ _ _ _ _ _ A B C D] new
                        // 2 [E F G H _ _ A B C D] new
                        //    . . . .     D . . .
                        let mut buf = Self::new();
                        copy_nonoverlapping(self, &mut buf, src_index, dst_index, dst_to_end);
                        copy_nonoverlapping(
                            self,
                            &mut buf,
                            src_index + dst_to_end,
                            0,
                            count - dst_to_end,
                        );
                        *self = buf;
                    }
                }
            },
            (true, true, false) => unsafe {
                hint::unreachable_unchecked();
            },
            (true, true, true) => unsafe {
                match count <= dst_to_src {
                    true => {
                        //    . . . .     S . . .
                        // 0 [E F G H _ _ A B C D]
                        // 1 [E E F G H _ A B C D]
                        // 2 [D E F G H _ A B C D]
                        // 3 [D E F G H _ A A B C]
                        //    . . . . .     D . .
                        self.copy_within(0, src_to_dst, count - src_to_end);
                        self.copy_within_nonoverlapping(dst_to_src, 0, src_to_dst);
                        self.copy_within(src_index, dst_index, dst_to_end);
                    }
                    false => {
                        //    . . . .     S . . .
                        // 0 [E F G H _ _ A B C D] old
                        // 1 [_ _ _ _ _ _ _ _ _ A] new
                        // 2 [B C D _ _ _ _ _ _ A] new
                        // 3 [B C D E F G H _ _ A] new
                        //    . . . . . . .     D
                        let mut buf = Self::new();
                        copy_nonoverlapping(self, &mut buf, src_index, dst_index, dst_to_end);
                        copy_nonoverlapping(self, &mut buf, src_index + dst_to_end, 0, src_to_dst);
                        copy_nonoverlapping(self, &mut buf, 0, src_to_dst, count - src_to_end);
                        *self = buf;
                    }
                }
            },
            (false, false, false) => unsafe {
                match count <= dst_to_src {
                    true => {
                        //              S . . .
                        // 0 [_ _ _ _ _ A B C D _]
                        // 1 [_ A B C D A B C D _]
                        //      D . . .
                        self.copy_within_nonoverlapping(src_index, dst_index, count);
                    }
                    false => {
                        //            S . . .
                        // 0 [_ _ _ _ A B C D _ _]
                        // 1 [_ A B C D B C D _ _]
                        //      D . . .
                        self.copy_within(src_index, dst_index, count);
                    }
                }
            },
            (false, false, true) => unsafe {
                hint::unreachable_unchecked();
            },
            (false, true, false) => unsafe {
                match (count <= src_to_dst, count <= dst_to_src) {
                    (true, true) => {
                        //    .             S . .
                        // 0 [D _ _ _ _ _ _ A B C]
                        // 1 [D A B C _ _ _ A B C]
                        // 2 [D A B C D _ _ A B C]
                        //      D . . .
                        self.copy_within_nonoverlapping(src_index, dst_index, src_to_end);
                        self.copy_within_nonoverlapping(
                            0,
                            dst_index + src_to_end,
                            count - src_to_end,
                        );
                    }
                    (true, false) => {
                        //    .             S . .
                        // 0 [D _ _ _ _ _ _ A B C]
                        // 1 [D _ _ _ _ A B C B C]
                        // 2 [D _ _ _ _ A B C D C]
                        //              D . . .
                        self.copy_within(src_index, dst_index, src_to_end);
                        self.copy_within_nonoverlapping(
                            0,
                            dst_index + src_to_end,
                            count - src_to_end,
                        );
                    }
                    (false, true) => {
                        //    . . .             S
                        // 0 [B C D _ _ _ _ _ _ A]
                        // 1 [B C B C D _ _ _ _ A]
                        // 2 [B A B C D _ _ _ _ A]
                        //      D . . .
                        self.copy_within(0, dst_index + src_to_end, count - src_to_end);
                        self.copy_within_nonoverlapping(src_index, dst_index, src_to_end);
                    }
                    (false, false) => {
                        //    . . . .     S . . .
                        // 0 [E F G H _ _ A B C D] old
                        // 1 [_ A B C D _ _ _ _ _] new
                        // 2 [_ A B C D E F G H _] new
                        //      D . . . . . . .
                        let mut buf = Self::new();
                        copy_nonoverlapping(self, &mut buf, src_index, dst_index, src_to_end);
                        copy_nonoverlapping(
                            self,
                            &mut buf,
                            0,
                            dst_index + src_to_end,
                            count - src_to_end,
                        );
                        *self = buf;
                    }
                }
            },
            (false, true, true) => unsafe {
                match count <= src_to_dst {
                    true => {
                        //    . . . . .     S . .
                        // 0 [D E F G H _ _ A B C]
                        // 1 [D E F G H _ A B C C]
                        // 2 [D E F G H _ A B C D]
                        // 3 [E F G H H _ A B C D]
                        //    . . . .     D . . .
                        self.copy_within(src_index, dst_index, src_to_end);
                        self.copy_within_nonoverlapping(0, src_to_dst, dst_to_src);
                        self.copy_within(dst_to_src, 0, count - dst_to_end);
                    }
                    false => {
                        //    . . . . . . .     S
                        // 0 [B C D E F G H _ _ A] old
                        // 1 [_ _ _ _ _ _ A _ _ _] new
                        // 1 [_ _ _ _ _ _ A B C D] new
                        // 1 [E F G H _ _ A B C D] new
                        //    . . . .     D . . .
                        let mut buf = Self::new();
                        copy_nonoverlapping(self, &mut buf, src_index, dst_index, src_to_end);
                        copy_nonoverlapping(self, &mut buf, 0, dst_index + src_to_end, dst_to_src);
                        copy_nonoverlapping(self, &mut buf, dst_to_src, 0, count - dst_to_end);
                        *self = buf;
                    }
                }
            },
        }
    }

    /// The caller must ensure that:
    ///
    /// - `index < N`.
    /// - `addend <= N`.
    pub(crate) const fn wrap_add(index: usize, addend: usize) -> usize {
        // Due to allocation limits, addition overflow is only possible when `T` is
        // zero-sized. In that case, the result may be at an incorrect location but
        // is still in bounds, which is sufficient for zero-sized types.
        Self::wrap_index(index.wrapping_add(addend))
    }

    /// The caller must ensure that:
    ///
    /// - `index < N`.
    /// - `subtrahend <= N`.
    pub(crate) const fn wrap_sub(index: usize, subtrahend: usize) -> usize {
        // Due to allocation limits, addition overflow is only possible when `T` is
        // zero-sized. In that case, the result may be at an incorrect location but
        // is still in bounds, which is sufficient for zero-sized types.
        Self::wrap_index(index.wrapping_sub(subtrahend).wrapping_add(N))
    }

    /// The caller must ensure that:
    ///
    /// - `index < 2 * N`.
    pub(crate) const fn wrap_index(index: usize) -> usize {
        if index < N { index } else { index - N }
    }
}

/// The caller must ensure that:
///
/// - `src_index < N`.
/// - `src_index + count <= N`.
/// - `dst_index < M`.
/// - `dst_index + count <= M`.
pub(crate) const unsafe fn copy_nonoverlapping<T, const N: usize, const M: usize>(
    src: &Buf<T, N>,
    dst: &mut Buf<T, M>,
    src_index: usize,
    dst_index: usize,
    count: usize,
) {
    let src_base = src.as_ptr();
    let dst_base = dst.as_mut_ptr();
    unsafe {
        let src = src_base.add(src_index);
        let dst = dst_base.add(dst_index);
        ptr::copy_nonoverlapping(src, dst, count);
    }
}

#[derive(Debug)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) len: usize,
}

impl From<Span> for Range<usize> {
    fn from(value: Span) -> Self {
        let start = value.start;
        let end = value.start + value.len;
        Range { start, end }
    }
}

pub(crate) trait BufIndex<T, const N: usize> {
    type Output: MaybeUninit + ?Sized;

    unsafe fn get_unchecked(self, buf: &Buf<T, N>) -> &Self::Output;

    unsafe fn get_unchecked_mut(self, buf: &mut Buf<T, N>) -> &mut Self::Output;
}

impl<T, const N: usize> BufIndex<T, N> for usize {
    type Output = mem::MaybeUninit<T>;

    unsafe fn get_unchecked(self, buf: &Buf<T, N>) -> &Self::Output {
        unsafe { buf.0.get_unchecked(self) }
    }

    unsafe fn get_unchecked_mut(self, buf: &mut Buf<T, N>) -> &mut Self::Output {
        unsafe { buf.0.get_unchecked_mut(self) }
    }
}

impl<T, const N: usize> BufIndex<T, N> for Span {
    type Output = [mem::MaybeUninit<T>];

    unsafe fn get_unchecked(self, buf: &Buf<T, N>) -> &Self::Output {
        let range = Range::from(self);
        unsafe { buf.0.get_unchecked(range) }
    }

    unsafe fn get_unchecked_mut(self, buf: &mut Buf<T, N>) -> &mut Self::Output {
        let range = Range::from(self);
        unsafe { buf.0.get_unchecked_mut(range) }
    }
}

impl<T, const N: usize> BufIndex<T, N> for Range<usize> {
    type Output = [mem::MaybeUninit<T>];

    unsafe fn get_unchecked(self, buf: &Buf<T, N>) -> &Self::Output {
        unsafe { buf.0.get_unchecked(self) }
    }

    unsafe fn get_unchecked_mut(self, buf: &mut Buf<T, N>) -> &mut Self::Output {
        unsafe { buf.0.get_unchecked_mut(self) }
    }
}

impl<T, const N: usize> BufIndex<T, N> for LegacyRange<usize> {
    type Output = [mem::MaybeUninit<T>];

    unsafe fn get_unchecked(self, buf: &Buf<T, N>) -> &Self::Output {
        unsafe { buf.0.get_unchecked(self) }
    }

    unsafe fn get_unchecked_mut(self, buf: &mut Buf<T, N>) -> &mut Self::Output {
        unsafe { buf.0.get_unchecked_mut(self) }
    }
}

pub(crate) trait MaybeUninit {
    type Init: ?Sized;

    unsafe fn assume_init_ref(&self) -> &Self::Init;

    unsafe fn assume_init_mut(&mut self) -> &mut Self::Init;

    unsafe fn assume_init_drop(&mut self);
}

impl<T> MaybeUninit for mem::MaybeUninit<T> {
    type Init = T;

    unsafe fn assume_init_ref(&self) -> &Self::Init {
        unsafe { self.assume_init_ref() }
    }

    unsafe fn assume_init_mut(&mut self) -> &mut Self::Init {
        unsafe { self.assume_init_mut() }
    }

    unsafe fn assume_init_drop(&mut self) {
        unsafe {
            self.assume_init_drop();
        }
    }
}

impl<T> MaybeUninit for [mem::MaybeUninit<T>] {
    type Init = [T];

    unsafe fn assume_init_ref(&self) -> &Self::Init {
        unsafe { self.assume_init_ref() }
    }

    unsafe fn assume_init_mut(&mut self) -> &mut Self::Init {
        unsafe { self.assume_init_mut() }
    }

    unsafe fn assume_init_drop(&mut self) {
        unsafe {
            self.assume_init_drop();
        }
    }
}

pub(crate) trait MaybeUninitExt: MaybeUninit<Init: Sized> {
    fn write(&mut self, value: Self::Init) -> &mut Self::Init;

    unsafe fn assume_init_read(&self) -> Self::Init;
}

impl<T> MaybeUninitExt for mem::MaybeUninit<T> {
    fn write(&mut self, value: Self::Init) -> &mut Self::Init {
        self.write(value)
    }

    unsafe fn assume_init_read(&self) -> Self::Init {
        unsafe { self.assume_init_read() }
    }
}
