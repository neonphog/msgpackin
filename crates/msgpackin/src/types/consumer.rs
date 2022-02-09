//! Data consumer traits for MessagePack Rust Encoding

use crate::*;

/// Trait representing a data consumer taking data synchronously
pub trait AsConsumerSync {
    /// Write data synchronously to this consumer
    fn write(&mut self, data: &[u8]) -> Result<()>;
}

/// Type alias for AsConsumerSync trait object
pub type DynConsumerSync<'lt> = Box<dyn AsConsumerSync + 'lt>;

#[cfg(not(feature = "std"))]
impl<'lt> From<&'lt mut Vec<u8>> for DynConsumerSync<'lt> {
    fn from(buf: &'lt mut Vec<u8>) -> Self {
        struct X<'lt>(&'lt mut Vec<u8>);
        impl<'lt> AsConsumerSync for X<'lt> {
            fn write(&mut self, data: &[u8]) -> Result<()> {
                self.0.extend_from_slice(data);
                Ok(())
            }
        }
        Box::new(X(buf))
    }
}

#[cfg(feature = "std")]
impl<'lt, W: ::std::io::Write + 'lt> From<W> for DynConsumerSync<'lt> {
    fn from(w: W) -> Self {
        struct X<W: ::std::io::Write>(W);
        impl<W: ::std::io::Write> AsConsumerSync for X<W> {
            fn write(&mut self, data: &[u8]) -> Result<()> {
                self.0.write_all(data)?;
                Ok(())
            }
        }
        Box::new(X(w))
    }
}

/// Trait representing a data consumer taking data asynchronously
pub trait AsConsumerAsync {
    /// Write data asynchronously to this consumer
    fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()>;
}

/// Type alias for AsConsumerSync trait object
pub type DynConsumerAsync<'lt> = Box<dyn AsConsumerAsync + 'lt>;

#[cfg(all(not(feature = "futures-io"), not(feature = "tokio")))]
impl<'lt> From<&'lt mut Vec<u8>> for DynConsumerAsync<'lt> {
    fn from(buf: &'lt mut Vec<u8>) -> Self {
        struct X<'lt>(&'lt mut Vec<u8>);
        impl<'lt> AsConsumerAsync for X<'lt> {
            fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()> {
                self.0.extend_from_slice(data);
                Box::pin(async move { Ok(()) })
            }
        }
        Box::new(X(buf))
    }
}

#[cfg(all(feature = "std", feature = "tokio"))]
impl<'lt, W: tokio::io::AsyncWrite + Unpin + 'lt> From<W>
    for DynConsumerAsync<'lt>
{
    fn from(w: W) -> Self {
        struct X<W: tokio::io::AsyncWrite + Unpin>(W);
        impl<W: tokio::io::AsyncWrite + Unpin> AsConsumerAsync for X<W> {
            fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()> {
                Box::pin(async move {
                    let w = write_all::WriteAll {
                        writer: &mut self.0,
                        buf: data,
                    };
                    w.await?;
                    Ok(())
                })
            }
        }
        Box::new(X(w))
    }
}

#[cfg(all(feature = "std", feature = "futures-io", not(feature = "tokio")))]
impl<'lt, W: futures_io::AsyncWrite + Unpin + 'lt> From<W>
    for DynConsumerAsync<'lt>
{
    fn from(w: W) -> Self {
        struct X<W: futures_io::AsyncWrite + Unpin>(W);
        impl<W: futures_io::AsyncWrite + Unpin> AsConsumerAsync for X<W> {
            fn write<'a>(&'a mut self, data: &'a [u8]) -> BoxFut<'a, ()> {
                Box::pin(async move {
                    let w = write_all::WriteAll {
                        writer: &mut self.0,
                        buf: data,
                    };
                    w.await?;
                    Ok(())
                })
            }
        }
        Box::new(X(w))
    }
}

// -- stolen from futures-util:
#[cfg(all(feature = "std", any(feature = "futures-io", feature = "tokio")))]
mod write_all {
    use super::*;

    macro_rules! ready {
        ($e:expr $(,)?) => {
            match $e {
                std::task::Poll::Ready(t) => t,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }
        };
    }

    pub struct WriteAll<'a, W: ?Sized> {
        pub writer: &'a mut W,
        pub buf: &'a [u8],
    }

    impl<W: ?Sized + Unpin> Unpin for WriteAll<'_, W> {}

    #[cfg(all(feature = "futures-io", not(feature = "tokio")))]
    impl<W: futures_io::AsyncWrite + ?Sized + Unpin> Future for WriteAll<'_, W> {
        type Output = std::io::Result<()>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let this = &mut *self;
            while !this.buf.is_empty() {
                use futures_io::AsyncWrite;
                let n = ready!(std::pin::Pin::new(&mut this.writer)
                    .poll_write(cx, this.buf))?;
                {
                    let (_, rest) =
                        mem::replace(&mut this.buf, &[]).split_at(n);
                    this.buf = rest;
                }
                if n == 0 {
                    return std::task::Poll::Ready(Err(
                        std::io::ErrorKind::WriteZero.into(),
                    ));
                }
            }

            std::task::Poll::Ready(Ok(()))
        }
    }

    #[cfg(feature = "tokio")]
    impl<W: tokio::io::AsyncWrite + ?Sized + Unpin> Future for WriteAll<'_, W> {
        type Output = std::io::Result<()>;

        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            let this = &mut *self;
            while !this.buf.is_empty() {
                use tokio::io::AsyncWrite;
                let n = ready!(std::pin::Pin::new(&mut this.writer)
                    .poll_write(cx, this.buf))?;
                {
                    let (_, rest) =
                        mem::replace(&mut this.buf, &[]).split_at(n);
                    this.buf = rest;
                }
                if n == 0 {
                    return std::task::Poll::Ready(Err(
                        std::io::ErrorKind::WriteZero.into(),
                    ));
                }
            }

            std::task::Poll::Ready(Ok(()))
        }
    }
}
