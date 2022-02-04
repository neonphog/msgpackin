//! Rust msgpack encoding / decoding / serialization library supporting no_std, no alloc, and serde

#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(all(feature = "std", feature = "serde", not(feature = "serde_std")))]
compile_error!("If features \"std\" and \"serde\" are enabled, feature \"serde_std\" must also be enabled. This workaround can be removed once weak dependency features are stable.");

// lib facade
mod lib {
    pub mod core {
        #[cfg(not(feature = "std"))]
        pub use core::*;
        #[cfg(feature = "std")]
        pub use std::*;
    }

    pub use self::core::fmt;
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

pub use msgpackin_core::num::Num;

#[cfg(all(not(feature = "std"), feature = "serde"))]
pub use ::serde::de::StdError;
#[cfg(feature = "std")]
pub use ::std::error::Error as StdError;
#[cfg(all(not(feature = "std"), not(feature = "serde")))]
pub use std_err::Error as StdError;

mod config;
pub use config::*;

mod error;
pub use error::*;

pub mod value;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
