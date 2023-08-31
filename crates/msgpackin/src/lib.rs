//! Msgpackin pure Rust MessagePack encoding / decoding library.
//!
//! Msgpackin supports `no_std`, but requires the `alloc` crate.
//! If you're looking for no alloc support, see the `msgpackin_core` crate.
//!
//! Msgpackin supports serde with the `serde` feature in either `no_std`
//! or `std` mode. If you want `std` mode, you must also enable the
//! `serde_std` feature, as a temporary workaround until weak dependency
//! features are stablized.
//!
//! ### Roadmap
//!
//! - [x] Value / ValueRef encoding and decoding without serde
//! - [x] zero-copy string/binary/ext data with `from_ref`
//! - [x] `no_std` serde support
//! - [x] `std::io::{Read, Write}` support in `std` mode
//! - [x] Async IO support via `futures-io` or `tokio` features
//! - [ ] recursion depth checking (the config is currently a stub)
//! - [ ] hooks for managed encoding / decoding of ext types
//!       (e.g. Timestamp (`-1`))
//! - [ ] benchmarking / optimization
//!
//! ### Features
//!
//! - `std` - enabled by default, pulls in the rust std library, enabling
//!           encoding and decoding via `std::io::{Read, Write}` traits
//! - `serde` - enables serialization / deserialization through the `serde`
//!             crate
//! - `futures-io` - enables async encoding and decoding through the futures
//!                  `io::{AsyncRead, AsyncWrite}` traits
//! - `tokio` - enables async encoding and decoding through the tokio
//!             `io::{AsyncRead, AsyncWrite}` traits
//!
//! ### `no_std` Example
//!
//! ```
//! use msgpackin::*;
//! let expect = Value::Map(vec![
//!     ("nil".into(), ().into()),
//!     ("bool".into(), true.into()),
//!     ("int".into(), (-42_i8).into()),
//!     ("bigInt".into(), u64::MAX.into()),
//!     ("float".into(), 3.141592653589793_f64.into()),
//!     ("str".into(), "hello".into()),
//!     ("ext".into(), Value::Ext(-42, b"ext-data".to_vec().into())),
//!     ("arr".into(), Value::Arr(vec!["one".into(), "two".into()])),
//! ]);
//! let encoded = expect.to_bytes().unwrap();
//! let decoded = ValueRef::from_ref(&encoded).unwrap();
//! assert_eq!(expect, decoded);
//! ```
//!
//! ### `std` Example
//!
//! ```
//! # #[cfg(feature = "std")]
//! # {
//! use msgpackin::*;
//! let expect = Value::Map(vec![("foo".into(), "bar".into())]);
//! let mut buf = Vec::new();
//!
//! {
//!     let writer: Box<dyn std::io::Write> = Box::new(&mut buf);
//!     expect.to_sync(writer).unwrap();
//! }
//!
//! let reader: Box<dyn std::io::Read> = Box::new(buf.as_slice());
//! let decoded = Value::from_sync(reader).unwrap();
//! assert_eq!(expect, decoded);
//! # }
//! ```
//!
//! ### Async Example
//!
//! ```
//! # #[cfg(feature = "tokio")]
//! # {
//! use msgpackin::*;
//! let expect = Value::Map(vec![("foo".into(), "bar".into())]);
//! let mut buf = Vec::new();
//!
//! {
//!     let writer: Box<dyn tokio::io::AsyncWrite + Unpin> = Box::new(&mut buf);
//!     futures::executor::block_on(async { expect.to_async(writer).await })
//!         .unwrap();
//! }
//!
//! let reader: Box<dyn tokio::io::AsyncRead + Unpin> =
//!     Box::new(buf.as_slice());
//! let decoded =
//!     futures::executor::block_on(async { Value::from_async(reader).await })
//!         .unwrap();
//! assert_eq!(expect, decoded);
//! # }
//! ```
//!
//! ### `serde` Example
//!
//! ```
//! # #[cfg(feature = "serde")]
//! # {
//! use msgpackin::*;
//! #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
//! struct X {
//!     pub nil: (),
//!     pub bool_: bool,
//!     pub int: i8,
//!     pub big_int: u64,
//!     pub float: f64,
//!     pub str_: String,
//!     pub arr: Vec<String>,
//! }
//!
//! let expect = X {
//!     nil: (),
//!     bool_: true,
//!     int: -42,
//!     big_int: u64::MAX,
//!     float: 3.141592653589793,
//!     str_: "hello".into(),
//!     arr: vec!["one".into(), "two".into()],
//! };
//!
//! let encoded = to_bytes(&expect).unwrap();
//! let decoded: X = from_sync(encoded.as_slice()).unwrap();
//! assert_eq!(expect, decoded);
//! # }
//! ```

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "serde")]
const EXT_STRUCT_NAME: &str = "_ExtStruct";

// lib facade
mod lib {
    pub mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
    }

    pub use self::core::fmt;
    pub use self::core::future::Future;
    pub use self::core::iter;
    pub use self::core::mem;
    pub use self::core::pin;
    pub use self::core::result;

    #[cfg(not(feature = "std"))]
    pub use alloc::vec::Vec;
    #[cfg(feature = "std")]
    pub use std::vec::Vec;

    #[cfg(not(feature = "std"))]
    pub use alloc::boxed::Box;
    #[cfg(feature = "std")]
    pub use std::boxed::Box;

    #[cfg(not(feature = "std"))]
    pub use alloc::borrow::Cow;
    #[cfg(feature = "std")]
    pub use std::borrow::Cow;

    #[cfg(not(feature = "std"))]
    pub use alloc::string::{String, ToString};
    #[cfg(feature = "std")]
    pub use std::string::{String, ToString};
}
pub(crate) use lib::*;

mod std_err {
    use crate::*;

    /// Stand-in Error Trait, given we are not including "std"
    /// (or the stand-in error trait from serde)
    pub trait Error: fmt::Debug + fmt::Display {
        /// The underlying cause of this error, if any.
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            None
        }
    }
}

/// Msgpackin Types
pub mod types {
    use super::*;

    #[cfg(all(not(feature = "std"), feature = "serde"))]
    pub use ::serde::de::StdError;
    #[cfg(feature = "std")]
    pub use ::std::error::Error as StdError;
    #[cfg(all(not(feature = "std"), not(feature = "serde")))]
    pub use std_err::Error as StdError;

    pub use msgpackin_core::num::Num;

    mod config;
    pub use config::*;

    mod error;
    pub use error::*;

    /// Type alias for a pinned future
    pub type BoxFut<'a, T> = pin::Pin<Box<dyn Future<Output = Result<T>> + 'a>>;

    pub mod consumer;
    pub mod producer;
}

use types::*;

#[cfg(feature = "serde")]
pub mod ser;

#[cfg(feature = "serde")]
pub use ser::{
    to_async, to_async_config, to_bytes, to_bytes_config, to_sync,
    to_sync_config,
};

#[cfg(feature = "serde")]
pub mod de;

#[cfg(feature = "serde")]
pub use de::{
    from_async, from_async_config, from_ref, from_ref_config, from_sync,
    from_sync_config,
};

pub mod value;

pub use value::Value;
pub use value::ValueRef;

#[cfg(test)]
mod test;
