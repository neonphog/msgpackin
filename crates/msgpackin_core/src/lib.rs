//! no_std, no alloc, MessagePack Rust encode / decode library code

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]

pub mod decode;
pub mod encode;

#[cfg(test)]
mod test;
