//! Msgpackin pure Rust MessagePack encoding / decoding library.
//!
//! This crate:
//! - supports `no_std`, but requires the `alloc` crate. If you're looking
//!   for no alloc support, see the `msgpackin_core` crate
//! - supports reading / writing to byte arrays / `Vec<u8>`s in `no_std` mode
//! - supports reading / writing to `std::io::{Read, Write}` in `std` mode
//! - supports async reading / writing with the optional `futures-io` feature
//! - supports async reading / writing with the optional `tokio` feature
//! - supports `serde` in `no_std` mode
//! - supports `serde` in `std` mode. (note, for now, you must also enable
//!   the `serde_std` feature as a workaround until weak dependency features)

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(all(feature = "std", feature = "serde", not(feature = "serde_std")))]
compile_error!("If features \"std\" and \"serde\" are enabled, feature \"serde_std\" must also be enabled. This workaround can be removed once weak dependency features are stable.");

#[cfg(all(feature = "futures-io", not(feature = "std")))]
compile_error!(
    "You cannot enable feature \"futures-io\" without also enabling \"std\""
);

#[cfg(all(feature = "tokio", not(feature = "std")))]
compile_error!(
    "You cannot enable feature \"tokio\" without also enabling \"std\""
);

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
