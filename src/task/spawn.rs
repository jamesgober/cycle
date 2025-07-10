//! Task spawning implementation

use std::future::Future;
use super::JoinHandle;

/// Spawn a new asynchronous task
pub fn spawn<F>(_future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // TODO: Implement actual spawning logic
    JoinHandle::new()
}
