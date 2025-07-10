//! CYCLE - Ultra-high performance async runtime

#![warn(missing_docs)]

use std::future::Future;
use std::sync::Arc;

pub mod runtime;
pub mod task;
pub mod scheduler;

#[cfg(feature = "net")]
pub mod net;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "sync")]
pub mod sync;

/// High-performance global runtime
static GLOBAL_RUNTIME: once_cell::sync::Lazy<Arc<runtime::Runtime>> = 
    once_cell::sync::Lazy::new(|| Arc::new(runtime::Runtime::new()));

/// Spawn a task on the global CYCLE runtime
pub fn spawn<F>(future: F) -> task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    GLOBAL_RUNTIME.spawn(future)
}

/// Block on a future using the global runtime
pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    GLOBAL_RUNTIME.block_on(future)
}

/// Get global runtime statistics
pub fn stats() -> runtime::RuntimeStatsSnapshot {
    GLOBAL_RUNTIME.stats()
}

/// Prelude module
pub mod prelude {
    pub use crate::{spawn, block_on, stats};
    pub use crate::runtime::Runtime;
    pub use crate::task::JoinHandle;
    
    #[cfg(feature = "net")]
    pub use crate::net::{TcpListener, TcpStream};
    
    #[cfg(feature = "time")]
    pub use crate::time::sleep;
    
    #[cfg(feature = "sync")]
    pub use crate::sync::{Mutex, RwLock};
    
    // Re-export standard time types
    pub use std::time::{Duration, Instant};
}
