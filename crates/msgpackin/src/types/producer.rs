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

#[cfg(not(feature = "std"))]
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

#[cfg(feature = "std")]
impl<'lt, R: ::std::io::Read + 'lt> From<R> for DynProducerSync<'lt> {
    fn from(r: R) -> Self {
        struct X<R: ::std::io::Read>(R, [u8; 4096]);
        impl<R: ::std::io::Read> AsProducerSync for X<R> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
            ) -> Result<Option<&'a [u8]>> {
                let Self(r, buf) = self;
                match r.read(&mut buf[..]) {
                    Ok(0) => Ok(None),
                    Ok(size) => Ok(Some(&buf[..size])),
                    Err(e) => Err(e.into()),
                }
            }
        }
        Box::new(X(r, [0; 4096]))
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

#[cfg(all(not(feature = "futures-io"), not(feature = "tokio")))]
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

#[cfg(all(feature = "std", feature = "futures-io", not(feature = "tokio")))]
impl<'lt, R: futures_io::AsyncRead + Unpin + 'lt> From<R> for DynProducerAsync<'lt> {
    fn from(r: R) -> Self {
        struct X<R: futures_io::AsyncRead + Unpin>(R, [u8; 4096]);
        impl<R: futures_io::AsyncRead + Unpin> AsProducerAsync for X<R> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
            ) -> BoxFut<'a, Option<&'a [u8]>> {
                Box::pin(async move {
                    let Self(r, buf) = self;
                    let r = read::Read {
                        reader: r,
                        buf,
                    };
                    match r.await {
                        Ok(0) => Ok(None),
                        Ok(size) => Ok(Some(&buf[..size])),
                        Err(e) => Err(e.into()),
                    }
                })
            }
        }
        Box::new(X(r, [0; 4096]))
    }
}

#[cfg(all(feature = "std", feature = "tokio"))]
impl<'lt, R: tokio::io::AsyncRead + Unpin + 'lt> From<R> for DynProducerAsync<'lt> {
    fn from(r: R) -> Self {
        struct X<R: tokio::io::AsyncRead + Unpin>(R, [u8; 4096]);
        impl<R: tokio::io::AsyncRead + Unpin> AsProducerAsync for X<R> {
            fn read_next<'a>(
                &'a mut self,
                _len_hint: u32,
            ) -> BoxFut<'a, Option<&'a [u8]>> {
                Box::pin(async move {
                    let Self(r, buf) = self;
                    let r = read::Read {
                        reader: r,
                        buf,
                    };
                    match r.await {
                        Ok(0) => Ok(None),
                        Ok(size) => Ok(Some(&buf[..size])),
                        Err(e) => Err(e.into()),
                    }
                })
            }
        }
        Box::new(X(r, [0; 4096]))
    }
}

// -- stolen from futures-util:
#[cfg(all(feature = "std", any(feature = "futures-io", feature = "tokio")))]
mod read {
    use super::*;

    pub struct Read<'a, R: ?Sized> {
        pub reader: &'a mut R,
        pub buf: &'a mut [u8],
    }

    impl<R: ?Sized + Unpin> Unpin for Read<'_, R> {}

    #[cfg(all(feature = "futures-io", not(feature = "tokio")))]
    impl<R: futures_io::AsyncRead + ?Sized + Unpin> Future for Read<'_, R> {
        type Output = std::io::Result<usize>;

        fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let this = &mut *self;
            use futures_io::AsyncRead;
            std::pin::Pin::new(&mut this.reader).poll_read(cx, this.buf)
        }
    }

    #[cfg(feature = "tokio")]
    impl<R: tokio::io::AsyncRead + ?Sized + Unpin> Future for Read<'_, R> {
        type Output = std::io::Result<usize>;

        fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
            let this = &mut *self;
            use tokio::io::AsyncRead;
            let mut rb = tokio::io::ReadBuf::new(this.buf);
            match std::pin::Pin::new(&mut this.reader).poll_read(cx, &mut rb) {
                std::task::Poll::Ready(Ok(_)) => std::task::Poll::Ready(Ok(rb.filled().len())),
                std::task::Poll::Pending => std::task::Poll::Pending,
                std::task::Poll::Ready(Err(e)) => std::task::Poll::Ready(Err(e)),
            }
        }
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
