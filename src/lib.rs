#![allow(clippy::missing_safety_doc)]
#![no_std]

pub mod deque;
pub mod string;
pub mod vec;

mod buf;

pub use self::deque::InlineDeque;
pub use self::vec::InlineVec;
