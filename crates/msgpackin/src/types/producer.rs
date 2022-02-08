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
pub trait AsProducerSync {
    /// Read the next chunk of data
    fn read_next<'a>(&'a mut self, len_hint: u32) -> Result<Option<&'a [u8]>>;
}

/// Type alias for AsProducerSync trait object
pub type DynProducerSync<'lt> = Box<dyn AsProducerSync + 'lt>;

impl<'lt> From<&'lt [u8]> for DynProducerSync<'lt> {
    fn from(buf: &'lt [u8]) -> Self {
        struct X<'lt>(&'lt [u8], bool);
        impl<'lt> AsProducerSync for X<'lt> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
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
        impl AsProducerSync for X {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
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
pub trait AsProducerAsync {
    /// Read the next chunk of data
    fn read_next<'a>(
        &'a mut self,
        len_hint: u32,
    ) -> BoxFut<'a, Option<&'a [u8]>>;
}

/// Type alias for AsProducerAsync trait object
pub type DynProducerAsync<'lt> = Box<dyn AsProducerAsync + 'lt>;

impl<'lt> From<&'lt [u8]> for DynProducerAsync<'lt> {
    fn from(buf: &'lt [u8]) -> Self {
        struct X<'lt>(&'lt [u8], bool);
        impl<'lt> AsProducerAsync for X<'lt> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
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
        impl AsProducerAsync for X {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
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

macro_rules! stub_wrap {
    ($($t:tt)*) => { $($t)* };
}

macro_rules! async_wrap {
    ($($t:tt)*) => { Box::pin(async move { $($t)* }) };
}

macro_rules! mk_decode_owned {
    (
        $id:ident,
        ($($prod:tt)*),
        ($($await:tt)*),
        ($($ret:tt)*),
        $wrap:ident,
    ) => {
        pub(crate) fn $id<'func, 'prod>(
            out: &'func mut Vec<OwnedToken>,
            dec: &'func mut msgpackin_core::decode::Decoder,
            prod: &'func mut $($prod)*,
            _config: &'func Config,
        ) -> $($ret)* {$wrap! {
            let mut len_type = msgpackin_core::decode::LenType::Bin;
            let mut buf = Vec::new();
            while let Some(data) = prod.read_next(dec.next_bytes_min())$($await)*? {
                for token in dec.parse(data) {
                    use msgpackin_core::decode::LenType;
                    use msgpackin_core::decode::Token::*;
                    match token {
                        Len(LenType::Arr, len) => out.push(OwnedToken::Arr(len)),
                        Len(LenType::Map, len) => out.push(OwnedToken::Map(len)),
                        Len(t, _len) => len_type = t,
                        Nil => out.push(OwnedToken::Nil),
                        Bool(b) => out.push(OwnedToken::Bool(b)),
                        Num(n) => out.push(OwnedToken::Num(n)),
                        BinCont(data, _) => buf.extend_from_slice(data),
                        Bin(data) => {
                            let owned_data = if buf.is_empty() {
                                data.to_vec().into_boxed_slice()
                            } else {
                                buf.extend_from_slice(data);
                                mem::take(&mut buf).into_boxed_slice()
                            };
                            match len_type {
                                LenType::Bin => out.push(OwnedToken::Bin(owned_data)),
                                LenType::Str => out.push(OwnedToken::Str(owned_data)),
                                LenType::Ext(t) => out.push(OwnedToken::Ext(t, owned_data)),
                                _ => unreachable!(), // is it?
                            }
                        }
                    }
                }
            }
            Ok(())
        }}
    };
}

mk_decode_owned!(
    priv_decode_owned_sync,
    (DynProducerSync<'prod>),
    (),
    (Result<()>),
    stub_wrap,
);

mk_decode_owned!(
    priv_decode_owned_async,
    (DynProducerAsync<'prod>),
    (.await),
    (BoxFut<'func, ()>),
    async_wrap,
);
