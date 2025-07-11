//! I/O reactor for event-driven networking

use mio::{Events, Poll, Registry, Token, Interest};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::task::Waker as TaskWaker;
use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;

/// Global reactor instance
pub static REACTOR: Lazy<Reactor> = Lazy::new(|| {
    Reactor::new().expect("Failed to create reactor")
});

/// I/O reactor for managing async I/O events
pub struct Reactor {
    registry: Arc<Registry>,
    wakers: Arc<Mutex<HashMap<Token, TaskWaker>>>,
    next_token: std::sync::atomic::AtomicUsize,
    shutdown: Arc<std::sync::atomic::AtomicBool>,
}

impl Reactor {
    /// Create a new reactor
    pub fn new() -> io::Result<Self> {
        let poll = Poll::new()?;
        let registry = Arc::new(poll.registry().try_clone()?);
        let wakers = Arc::new(Mutex::new(HashMap::new()));
        let next_token = std::sync::atomic::AtomicUsize::new(1);
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        // Start reactor thread
        let poll_clone = poll;
        let wakers_clone = wakers.clone();
        let shutdown_clone = shutdown.clone();
        
        thread::spawn(move || {
            Self::run_event_loop(poll_clone, wakers_clone, shutdown_clone);
        });
        
        Ok(Self {
            registry,
            wakers,
            next_token,
            shutdown,
        })
    }
    
    /// Register an I/O source
    pub fn register<S>(&self, source: &mut S, interest: Interest) -> io::Result<Token>
    where
        S: mio::event::Source + ?Sized,
    {
        let token_value = self.next_token.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let token = Token(token_value);
        
        self.registry.register(source, token, interest)?;
        Ok(token)
    }
    
    /// Reregister an I/O source
    pub fn reregister<S>(&self, source: &mut S, token: Token, interest: Interest) -> io::Result<()>
    where
        S: mio::event::Source + ?Sized,
    {
        self.registry.reregister(source, token, interest)
    }
    
    /// Deregister an I/O source
    pub fn deregister<S>(&self, source: &mut S) -> io::Result<()>
    where
        S: mio::event::Source + ?Sized,
    {
        self.registry.deregister(source)
    }
    
    /// Register a task waker for I/O readiness
    pub fn register_waker(&self, token: Token, waker: TaskWaker) {
        self.wakers.lock().unwrap().insert(token, waker);
    }

    /// Wait for I/O readiness
    pub fn wait_for_io(token: Token) -> impl std::future::Future<Output = io::Result<()>> {
        struct IoFuture {
            token: Token,
            registered: bool,
        }
        
        impl std::future::Future for IoFuture {
            type Output = io::Result<()>;
            
            fn poll(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if !self.registered {
                    REACTOR.register_waker(self.token, cx.waker().clone());
                    self.registered = true;
                }
                std::task::Poll::Pending
            }
        }
        
        IoFuture {
            token,
            registered: false,
        }
    }
    
    /// Run the event loop
    fn run_event_loop(
        mut poll: Poll,
        wakers: Arc<Mutex<HashMap<Token, TaskWaker>>>,
        shutdown: Arc<std::sync::atomic::AtomicBool>,
    ) {
        let mut events = Events::with_capacity(1024);
        
        while !shutdown.load(std::sync::atomic::Ordering::Acquire) {
            // Poll for events with timeout
            if let Err(_) = poll.poll(&mut events, Some(Duration::from_millis(10))) {
                continue;
            }
            
            // Process events
            for event in events.iter() {
                let token = event.token();
                
                // Wake the associated task
                if let Some(waker) = wakers.lock().unwrap().remove(&token) {
                    waker.wake();
                }
            }
        }
    }
    
    /// Shutdown the reactor
    pub fn shutdown(&self) {
        self.shutdown.store(true, std::sync::atomic::Ordering::Release);
    }
}

/// Extension method for reactor access
impl Reactor {
    /// Access reactor in a thread-local context
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Reactor) -> R,
    {
        f(&*REACTOR)
    }
}
