//! CYCLE - Ultra-high performance async runtime
//! 
//! Fast task cycling with zero-overhead scheduling, lock-free architecture, 
//! and built-in fault tolerance.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod runtime;
pub mod task;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "sync")]
pub mod sync;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "fs")]
pub mod fs;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "circuit-breaker")]
pub mod fault_tolerance;

/// Convenience re-exports for common types
pub mod prelude {
    pub use crate::runtime::{Runtime, Builder};
    pub use crate::task::{spawn, JoinHandle};
    
    #[cfg(feature = "net")]
    pub use crate::net::{TcpListener, TcpStream, UdpSocket};
    
    #[cfg(feature = "time")]
    pub use crate::time::{sleep, interval};
    
    #[cfg(feature = "sync")]
    pub use crate::sync::{Mutex, RwLock, mpsc, oneshot};
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Spawn a new asynchronous task
pub use task::spawn;

/// Block on a future until completion
pub use runtime::block_on;
