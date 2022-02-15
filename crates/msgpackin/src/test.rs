mod no_std_tests;

#[cfg(feature = "std")]
mod std_tests;

#[cfg(feature = "tokio")]
mod async_tests;

#[cfg(feature = "serde")]
mod serde_tests;
