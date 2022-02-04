//! Data provider traits for MessagePack Rust Decoding

// come on clippy... sometimes these help make more sense of things
#![allow(clippy::needless_lifetimes)]

use crate::*;

/// Trait representing a data provider that can provide
/// the full data in one call
pub trait AsProducerComplete<'buf> {
    /// Read all the data at once
    fn read_all(&mut self) -> Result<&'buf [u8]>;
}

/// Type alias for AsProducerComplete trait object
pub type DynProducerComplete<'buf> = Box<dyn AsProducerComplete<'buf> + 'buf>;

impl<'buf> From<&'buf [u8]> for DynProducerComplete<'buf> {
    fn from(buf: &'buf [u8]) -> Self {
        struct X<'buf>(&'buf [u8]);
        impl<'buf> AsProducerComplete<'buf> for X<'buf> {
            fn read_all(&mut self) -> Result<&'buf [u8]> {
                Ok(self.0)
            }
        }
        Box::new(X(buf))
    }
}

/// Trait representing a data provider that provides data in synchronous chunks
pub trait AsProducerSync<'lt> {
    /// Read the next chunk of data
    fn read_next<'a>(&'a mut self, len_hint: usize)
        -> Result<Option<&'a [u8]>>;
}

/// Type alias for AsProducerSync trait object
pub type DynProducerSync<'lt> = Box<dyn AsProducerSync<'lt> + 'lt>;

impl<'lt> From<&'lt [u8]> for DynProducerSync<'lt> {
    fn from(buf: &'lt [u8]) -> Self {
        struct X<'lt>(&'lt [u8], bool);
        impl<'lt> AsProducerSync<'lt> for X<'lt> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: usize,
            ) -> Result<Option<&'a [u8]>> {
                if self.1 {
                    self.1 = false;
                    Ok(Some(self.0))
                } else {
                    Ok(None)
                }
            }
        }
        Box::new(X(buf, true))
    }
}

impl From<Vec<u8>> for DynProducerSync<'_> {
    fn from(buf: Vec<u8>) -> Self {
        struct X(Vec<u8>, bool);
        impl<'lt> AsProducerSync<'lt> for X {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: usize,
            ) -> Result<Option<&'a [u8]>> {
                if self.1 {
                    self.1 = false;
                    Ok(Some(self.0.as_slice()))
                } else {
                    Ok(None)
                }
            }
        }
        Box::new(X(buf, true))
    }
}

/// Trait representing a data provider that provides data in async chunks
pub trait AsProducerAsync<'lt> {
    /// Read the next chunk of data
    fn read_next<'a>(
        &'a mut self,
        len_hint: usize,
    ) -> BoxFut<'a, Option<&'a [u8]>>;
}

/// Type alias for AsProducerAsync trait object
pub type DynProducerAsync<'lt> = Box<dyn AsProducerAsync<'lt> + 'lt>;

impl<'lt> From<&'lt [u8]> for DynProducerAsync<'lt> {
    fn from(buf: &'lt [u8]) -> Self {
        struct X<'lt>(&'lt [u8], bool);
        impl<'lt> AsProducerAsync<'lt> for X<'lt> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: usize,
            ) -> BoxFut<'a, Option<&'a [u8]>> {
                Box::pin(async move {
                    if self.1 {
                        self.1 = false;
                        Ok(Some(self.0))
                    } else {
                        Ok(None)
                    }
                })
            }
        }
        Box::new(X(buf, true))
    }
}

impl From<Vec<u8>> for DynProducerAsync<'_> {
    fn from(buf: Vec<u8>) -> Self {
        struct X(Vec<u8>, bool);
        impl<'lt> AsProducerAsync<'lt> for X {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: usize,
            ) -> BoxFut<'a, Option<&'a [u8]>> {
                Box::pin(async move {
                    if self.1 {
                        self.1 = false;
                        Ok(Some(self.0.as_slice()))
                    } else {
                        Ok(None)
                    }
                })
            }
        }
        Box::new(X(buf, true))
    }
}
