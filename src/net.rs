//! High-performance networking with real async I/O

use crate::io::{AsyncRead, AsyncWrite};
use crate::reactor::Reactor;
use std::io::{self, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::pin::Pin;
use std::task::{Context, Poll};
use mio::net::{TcpListener as MioTcpListener, TcpStream as MioTcpStream, UdpSocket as MioUdpSocket};
use mio::{Interest, Token};

/// High-performance TCP listener
pub struct TcpListener {
    inner: MioTcpListener,
    token: Token,
}

/// High-performance TCP stream
pub struct TcpStream {
    inner: MioTcpStream,
    token: Token,
    read_ready: bool,
    write_ready: bool,
}

/// High-performance UDP socket
pub struct UdpSocket {
    inner: MioUdpSocket,
    token: Token,
}

impl TcpListener {
    /// Bind to an address with real async I/O
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr.to_socket_addrs()?.next()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "No address provided"))?;
        
        let socket = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        socket.listen(1024)?;
        
        let std_listener: std::net::TcpListener = socket.into();
        let mut listener = MioTcpListener::from_std(std_listener);
        let token = Reactor::with(|reactor| reactor.register(&mut listener, Interest::READABLE))?;
        Ok(Self {
            inner: listener,
            token,
        })
    }
    
    /// Accept a new connection with proper async handling
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        loop {
            match self.inner.accept() {
                Ok((mut stream, addr)) => {
                    let token = Reactor::with(|reactor| {
                        reactor.register(&mut stream, Interest::READABLE | Interest::WRITABLE)
                    })?;
                    
                    return Ok((TcpStream {
                        inner: stream,
                        token,
                        read_ready: false,
                        write_ready: false,
                    }, addr));
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // Wait for readiness
                    let token = self.token;
                    let fut = Reactor::with(move |_reactor| {
                        Reactor::wait_for_io(token)
                    });
                    fut.await?;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

impl TcpStream {
    /// Connect to an address with real async I/O
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr.to_socket_addrs()?.next()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "No address provided"))?;
        
        let socket = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        
        socket.set_nonblocking(true)?;
        
        // Start connection
        match socket.connect(&addr.into()) {
            Ok(()) => {}
            Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::NotConnected => {}
            Err(e) => return Err(e),
        }
        
        let socket_std = socket.into();
        let mut stream = MioTcpStream::from_std(socket_std);
        let token = Reactor::with(|reactor| {
            reactor.register(&mut stream, Interest::READABLE | Interest::WRITABLE)
        })?;
        // Wait for connection to complete
        let fut = Reactor::with(|_reactor| {
            Reactor::wait_for_io(token)
        });
        fut.await?;
        Reactor::wait_for_io(token).await?;
        
        Ok(Self {
            inner: stream,
            token,
            read_ready: false,
            write_ready: false,
        })
    }
    /// Get local address
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.inner.local_addr()
    }
    
    /// Get peer address
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.inner.peer_addr()
    }
    
    /// Shutdown the connection
    pub fn shutdown(&self, how: std::net::Shutdown) -> io::Result<()> {
        self.inner.shutdown(how)
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.read(buf) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    if !self.read_ready {
                        // Register for readiness notification
                        Reactor::with(|reactor| reactor.register_waker(self.token, cx.waker().clone()));
                        self.read_ready = false;
                        return Poll::Pending;
                    }
                    self.read_ready = false;
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        loop {
            match self.inner.write(buf) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    if !self.write_ready {
                        Reactor::with(|reactor| reactor.register_waker(self.token, cx.waker().clone()));
                        return Poll::Pending;
                    }
                    self.write_ready = false;
                }
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
    
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.shutdown(std::net::Shutdown::Write)?;
        Poll::Ready(Ok(()))
    }
}

impl UdpSocket {
    /// Bind UDP socket
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let addr = addr.to_socket_addrs()?.next()
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "No address provided"))?;
        
        let socket = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;
        
        socket.set_reuse_address(true)?;
        socket.set_nonblocking(true)?;
        socket.bind(&addr.into())?;
        
        let mut socket = MioUdpSocket::from_std(socket.into());
        let token = Reactor::with(|reactor| {
            reactor.register(&mut socket, Interest::READABLE | Interest::WRITABLE)
        })?;
        
        Ok(Self {
            inner: socket,
            token,
        })
    }
    
    /// Send data to a specific address
    pub async fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        loop {
            match self.inner.send_to(buf, addr) {
                Ok(n) => return Ok(n),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    Reactor::wait_for_io(self.token).await?;
                }
                Err(e) => return Err(e),
            }
        }
    }
    
    /// Receive data from any address
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        loop {
            match self.inner.recv_from(buf) {
                Ok((n, addr)) => return Ok((n, addr)),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    Reactor::wait_for_io(self.token).await?;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

// Implement std::io traits for TcpStream
use std::io::{Read, Write};

impl Read for TcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for TcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
