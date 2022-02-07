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

pub(crate) enum OwnedToken {
    Bin(Box<[u8]>),
    Str(Box<[u8]>),
    Ext(i8, Box<[u8]>),
    Arr(u32),
    Map(u32),
    Nil,
    Bool(bool),
    Num(Num),
}

pub(crate) struct OwnedDecoder {
    dec: msgpackin_core::decode::Decoder,
    len_type: msgpackin_core::decode::LenType,
    buf: Vec<u8>,
}

impl Default for OwnedDecoder {
    fn default() -> Self {
        Self {
            dec: msgpackin_core::decode::Decoder::new(),
            len_type: msgpackin_core::decode::LenType::Bin,
            buf: Vec::new(),
        }
    }
}

impl OwnedDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_bytes_min(&self) -> u32 {
        self.dec.next_bytes_min()
    }

    pub fn parse(&mut self, data: &[u8]) -> Vec<OwnedToken> {
        let Self { dec, len_type, buf } = self;
        let mut tokens = Vec::new();
        for token in dec.parse(data) {
            use msgpackin_core::decode::LenType;
            use msgpackin_core::decode::Token::*;
            match token {
                Len(LenType::Arr, len) => tokens.push(OwnedToken::Arr(len)),
                Len(LenType::Map, len) => tokens.push(OwnedToken::Map(len)),
                Len(t, len) => *len_type = t,
                Nil => tokens.push(OwnedToken::Nil),
                Bool(b) => tokens.push(OwnedToken::Bool(b)),
                Num(n) => tokens.push(OwnedToken::Num(n)),
                BinCont(data, _) => buf.extend_from_slice(data),
                Bin(data) => {
                    let owned_data = if buf.is_empty() {
                        data.to_vec().into_boxed_slice()
                    } else {
                        buf.extend_from_slice(data);
                        mem::replace(buf, Vec::new()).into_boxed_slice()
                    };
                    match len_type {
                        LenType::Bin => tokens.push(OwnedToken::Bin(owned_data)),
                        LenType::Str => tokens.push(OwnedToken::Str(owned_data)),
                        LenType::Ext(t) => tokens.push(OwnedToken::Ext(*t, owned_data)),
                        _ => unreachable!(), // is it?
                    }
                }
            }
        }
        tokens
    }
}
