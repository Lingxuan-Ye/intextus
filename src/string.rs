use crate::buf;
use crate::buf::Buf;
use crate::error::StringError;
use crate::vec::InlineVec;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::fmt;
use core::ops::{Deref, DerefMut, Index, IndexMut};
use core::slice::SliceIndex;

mod convert;

#[cfg(feature = "serde")]
mod serde;

#[derive(Clone, Default, Hash, Eq, Ord)]
pub struct InlineString<const N: usize> {
    vec: InlineVec<u8, N>,
}

impl<const N: usize> InlineString<N> {
    pub const fn new() -> Self {
        let vec = InlineVec::new();
        Self { vec }
    }

    pub fn from_utf8(bytes: &[u8]) -> Result<Self, StringError> {
        let string = str::from_utf8(bytes).map_err(StringError::Utf8Error)?;
        let mut result = Self::new();
        result.push_str(string)?;
        Ok(result)
    }

    pub const unsafe fn from_utf8_unchecked(bytes: InlineVec<u8, N>) -> Self {
        Self { vec: bytes }
    }

    pub fn from_utf8_lossy(bytes: &[u8]) -> Result<Self, StringError> {
        let mut result = Self::new();
        for chunk in bytes.utf8_chunks() {
            let valid = chunk.valid();
            result.push_str(valid)?;
            if !chunk.invalid().is_empty() {
                result.push(char::REPLACEMENT_CHARACTER)?;
            }
        }
        Ok(result)
    }

    pub fn from_utf16(bytes: &[u16]) -> Result<Self, StringError> {
        let mut result = Self::new();
        for char in char::decode_utf16(bytes.iter().cloned()) {
            let char = char.map_err(StringError::Utf16Error)?;
            result.push(char)?;
        }
        Ok(result)
    }

    pub fn from_utf16_lossy(bytes: &[u16]) -> Result<Self, StringError> {
        let mut result = Self::new();
        for char in char::decode_utf16(bytes.iter().cloned()) {
            let char = char.unwrap_or(char::REPLACEMENT_CHARACTER);
            result.push(char)?;
        }
        Ok(result)
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn len(&self) -> usize {
        self.vec.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub const fn is_full(&self) -> bool {
        self.vec.is_full()
    }

    pub const fn as_str(&self) -> &str {
        let slice = self.vec.as_slice();
        unsafe { str::from_utf8_unchecked(slice) }
    }

    pub const fn as_mut_str(&mut self) -> &mut str {
        let slice = self.vec.as_mut_slice();
        unsafe { str::from_utf8_unchecked_mut(slice) }
    }

    pub const fn as_bytes(&self) -> &[u8] {
        self.vec.as_slice()
    }

    pub const unsafe fn as_mut_vec(&mut self) -> &mut InlineVec<u8, N> {
        &mut self.vec
    }

    pub fn into_bytes(self) -> InlineVec<u8, N> {
        self.vec
    }

    pub const fn push(&mut self, char: char) -> Result<(), StringError> {
        let len = self.vec.len();
        let char_len = char.len_utf8();
        if N - len < char_len {
            let error = StringError::capacity_overflow();
            return Err(error);
        }
        unsafe {
            self.write(len, char);
            self.vec.set_len(len + char_len);
        }
        Ok(())
    }

    pub const fn push_str(&mut self, string: &str) -> Result<(), StringError> {
        let len = self.vec.len();
        let string_len = string.len();
        if N - len < string_len {
            let error = StringError::capacity_overflow();
            return Err(error);
        }
        unsafe {
            self.vec
                .as_mut_ptr()
                .add(len)
                .copy_from_nonoverlapping(string.as_ptr(), string_len);
            self.vec.set_len(len + string_len);
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<char> {
        let char = self.as_str().chars().next_back()?;
        let len = self.vec.len();
        let char_len = char.len_utf8();
        unsafe {
            self.vec.set_len(len - char_len);
        }
        Some(char)
    }

    pub const fn insert(&mut self, index: usize, char: char) -> Result<(), StringError> {
        if !self.as_str().is_char_boundary(index) {
            let error = StringError::not_char_boundary(index);
            return Err(error);
        }
        let len = self.vec.len();
        let char_len = char.len_utf8();
        if N - len < char_len {
            let error = StringError::capacity_overflow();
            return Err(error);
        }
        if index != len {
            unsafe {
                self.vec
                    .buf_mut()
                    .copy_within(index, index + char_len, len - index);
            }
        }
        unsafe {
            self.write(index, char);
            self.vec.set_len(len + char_len);
        }
        Ok(())
    }

    pub const fn insert_str(&mut self, index: usize, string: &str) -> Result<(), StringError> {
        if !self.as_str().is_char_boundary(index) {
            let error = StringError::not_char_boundary(index);
            return Err(error);
        }
        let len = self.vec.len();
        let string_len = string.len();
        if N - len < string_len {
            let error = StringError::capacity_overflow();
            return Err(error);
        }
        if index != len {
            unsafe {
                self.vec
                    .buf_mut()
                    .copy_within(index, index + string_len, len - index);
            }
        }
        unsafe {
            self.vec
                .as_mut_ptr()
                .add(index)
                .copy_from_nonoverlapping(string.as_ptr(), string_len);
            self.vec.set_len(len + string_len);
        }
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Option<char> {
        let char = self.as_str().get(index..)?.chars().next()?;
        let len = self.vec.len();
        let char_len = char.len_utf8();
        let new_len = len - char_len;
        if index != new_len {
            unsafe {
                self.vec
                    .buf_mut()
                    .copy_within(index + char_len, index, new_len - index);
            }
        }
        unsafe {
            self.vec.set_len(new_len);
        }
        Some(char)
    }

    pub fn truncate(&mut self, len: usize) -> Result<(), StringError> {
        if len > self.vec.len() {
            return Ok(());
        }
        if !self.as_str().is_char_boundary(len) {
            let error = StringError::not_char_boundary(len);
            return Err(error);
        }
        self.vec.truncate(len);
        Ok(())
    }

    pub fn split_off(&mut self, at: usize) -> Result<Self, StringError> {
        if !self.as_str().is_char_boundary(at) {
            let error = StringError::not_char_boundary(at);
            return Err(error);
        }
        let mut result = Self::new();
        let dst_len = self.vec.len() - at;
        unsafe {
            self.vec.set_len(at);
            let src_index = at;
            let dst_index = 0;
            let count = dst_len;
            buf::copy_nonoverlapping(
                self.vec.buf(),
                result.vec.buf_mut(),
                src_index,
                dst_index,
                count,
            );
            result.vec.set_len(dst_len);
        }
        Ok(result)
    }

    pub fn clear(&mut self) {
        self.vec.clear();
    }

    pub(crate) const fn buf(&self) -> &Buf<u8, N> {
        self.vec.buf()
    }

    /// # Safety
    ///
    /// The caller must ensure that:
    ///
    /// - `index` is a char boundary.
    /// - `index + char.len_utf8() <= N`.
    pub(crate) const unsafe fn write(&mut self, index: usize, char: char) {
        let code = char as u32;
        let dst = unsafe { self.vec.as_mut_ptr().add(index) };
        match char.len_utf8() {
            1 => unsafe {
                *dst = code as u8;
            },
            2 => unsafe {
                *dst = (code >> 6 | 0b1100_0000) as u8;
                *dst.add(1) = (code & 0b0011_1111 | 0b1000_0000) as u8;
            },
            3 => unsafe {
                *dst = (code >> 12 | 0b1110_0000) as u8;
                *dst.add(1) = (code >> 6 & 0b0011_1111 | 0b1000_0000) as u8;
                *dst.add(2) = (code & 0b0011_1111 | 0b1000_0000) as u8;
            },
            _ => unsafe {
                *dst = (code >> 18 | 0b1111_0000) as u8;
                *dst.add(1) = (code >> 12 & 0b0011_1111 | 0b1000_0000) as u8;
                *dst.add(2) = (code >> 6 & 0b0011_1111 | 0b1000_0000) as u8;
                *dst.add(3) = (code & 0b0011_1111 | 0b1000_0000) as u8;
            },
        }
    }
}

impl<const N: usize> fmt::Debug for InlineString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<const N: usize> fmt::Display for InlineString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl<const N: usize> Deref for InlineString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> DerefMut for InlineString<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_str()
    }
}

impl<const N: usize> AsRef<str> for InlineString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<[u8]> for InlineString<N> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const N: usize> AsMut<str> for InlineString<N> {
    fn as_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl<const N: usize> Borrow<str> for InlineString<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> BorrowMut<str> for InlineString<N> {
    fn borrow_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl<const N: usize, const M: usize> PartialEq<InlineString<M>> for InlineString<N> {
    fn eq(&self, other: &InlineString<M>) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<const N: usize> PartialEq<str> for InlineString<N> {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl<const N: usize> PartialEq<InlineString<N>> for str {
    fn eq(&self, other: &InlineString<N>) -> bool {
        self.eq(other.as_str())
    }
}

impl<const N: usize, const M: usize> PartialOrd<InlineString<M>> for InlineString<N> {
    fn partial_cmp(&self, other: &InlineString<M>) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<const N: usize, I> Index<I> for InlineString<N>
where
    I: SliceIndex<str>,
{
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_str().index(index)
    }
}

impl<const N: usize, I> IndexMut<I> for InlineString<N>
where
    I: SliceIndex<str>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.as_mut_str().index_mut(index)
    }
}
