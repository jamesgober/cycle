//! High-performance networking primitives

use std::io;
use std::net::SocketAddr;

/// TCP listener for accepting connections
pub struct TcpListener {
    inner: std::net::TcpListener,
}

/// TCP stream for network communication
pub struct TcpStream {
    inner: std::net::TcpStream,
}

impl TcpListener {
    /// Bind to an address
    pub async fn bind<A: std::net::ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let listener = std::net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        
        Ok(Self { inner: listener })
    }
    
    /// Accept a new connection
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        loop {
            match self.inner.accept() {
                Ok((stream, addr)) => {
                    stream.set_nonblocking(true)?;
                    return Ok((TcpStream { inner: stream }, addr));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Yield to allow other tasks to run
                    crate::time::yield_now().await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl TcpStream {
    /// Connect to an address
    pub async fn connect<A: std::net::ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let stream = std::net::TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        
        Ok(Self { inner: stream })
    }
}

// TODO: Implement AsyncRead/AsyncWrite traits
