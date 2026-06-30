#![allow(clippy::missing_safety_doc)]
#![no_std]

pub mod vec;

mod buf;

pub use self::vec::InlineVec;
