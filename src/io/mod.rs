//! Async I/O traits and utilities

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

/// Async read trait
pub trait AsyncRead {
    /// Poll read
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>>;
}

/// Async write trait  
pub trait AsyncWrite {
    /// Poll write
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>>;
    
    /// Poll flush
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
    
    /// Poll shutdown
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
}
/// Async read extension methods
pub trait AsyncReadExt: AsyncRead {
    /// Read data
    fn read(&mut self, _buf: &mut [u8]) -> impl Future<Output = io::Result<usize>> + '_
    where
        Self: Unpin,
    {
        async move {
            // TODO: Implement read method
            todo!("AsyncReadExt::read not implemented")
        }
    }
}

/// Async write extension methods
pub trait AsyncWriteExt: AsyncWrite {
    /// Write all data
    fn write_all(&mut self, _buf: &[u8]) -> impl Future<Output = io::Result<()>> + '_
    where
        Self: Unpin,
    {
        async move {
            // TODO: Implement write_all method
            todo!("AsyncWriteExt::write_all not implemented")
        }
    }
}

impl<T: AsyncRead> AsyncReadExt for T {}
impl<T: AsyncWrite> AsyncWriteExt for T {}
