//! Data consumer traits for MessagePack Rust Encoding

use crate::*;

/// Trait representing a data consumer taking data synchronously
pub trait AsConsumerSync<'lt> {
    /// Write data synchronously to this consumer
    fn write<'a>(&'a mut self, data: &'a [u8]) -> Result<()>;
}

/// Type alias for AsConsumerSync trait object
pub type DynConsumerSync<'lt> = Box<dyn AsConsumerSync<'lt> + 'lt>;

impl<'lt> From<&'lt mut Vec<u8>> for DynConsumerSync<'lt> {
    fn from(buf: &'lt mut Vec<u8>) -> Self {
        struct X<'lt>(&'lt mut Vec<u8>);
        impl<'lt> AsConsumerSync<'lt> for X<'lt> {
            fn write<'a>(&'a mut self, data: &'a [u8]) -> Result<()> {
                self.0.extend_from_slice(data);
                Ok(())
            }
        }
        Box::new(X(buf))
    }
}

/// Trait representing a data consumer taking data asynchronously
pub trait AsConsumerAsync<'lt> {
    /// Write data asynchronously to this consumer
    fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()>;
}

/// Type alias for AsConsumerSync trait object
pub type DynConsumerAsync<'lt> = Box<dyn AsConsumerAsync<'lt> + 'lt>;

impl<'lt> From<&'lt mut Vec<u8>> for DynConsumerAsync<'lt> {
    fn from(buf: &'lt mut Vec<u8>) -> Self {
        struct X<'lt>(&'lt mut Vec<u8>);
        impl<'lt> AsConsumerAsync<'lt> for X<'lt> {
            fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()> {
                self.0.extend_from_slice(data);
                Box::pin(async move { Ok(()) })
            }
        }
        Box::new(X(buf))
    }
}
