//! Task management and join handles

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

/// Join handle for spawned tasks
pub struct JoinHandle<T> {
    control: Arc<TaskControl<T>>,
}

/// Task control for managing task lifecycle
pub struct TaskControl<T> {
    result: Mutex<Option<Result<T, JoinError>>>,
    completed: std::sync::atomic::AtomicBool,
}

/// Join error
#[derive(Debug, Clone)]
pub struct JoinError {
    message: String,
}

impl<T> JoinHandle<T> {
    /// Create new join handle
    pub fn new(control: Arc<TaskControl<T>>) -> Self {
        Self { control }
    }
    
    /// Try to get result without blocking
    pub fn try_result(&self) -> Option<Result<T, JoinError>> {
        if self.control.completed.load(std::sync::atomic::Ordering::Acquire) {
            self.control.result.lock().unwrap().take()
        } else {
            None
        }
    }
    
    /// Check if task is completed
    pub fn is_finished(&self) -> bool {
        self.control.completed.load(std::sync::atomic::Ordering::Acquire)
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.try_result() {
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}

impl<T> TaskControl<T> {
    /// Create new task control
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            result: Mutex::new(None),
            completed: std::sync::atomic::AtomicBool::new(false),
        })
    }
    
    /// Complete the task with result
    pub fn complete(&self, result: Result<T, JoinError>) {
        *self.result.lock().unwrap() = Some(result);
        self.completed.store(true, std::sync::atomic::Ordering::Release);
    }
}

impl JoinError {
    /// Create new join error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JoinError {}
