//! Synchronization primitives

/// Async mutex
pub struct Mutex<T> {
    inner: parking_lot::Mutex<T>,
}

/// Async RwLock
pub struct RwLock<T> {
    inner: parking_lot::RwLock<T>,
}

impl<T> Mutex<T> {
    /// Create new mutex
    pub fn new(value: T) -> Self {
        Self {
            inner: parking_lot::Mutex::new(value),
        }
    }
    
    /// Lock the mutex
    pub async fn lock(&self) -> parking_lot::MutexGuard<'_, T> {
        self.inner.lock()
    }
}

impl<T> RwLock<T> {
    /// Create new RwLock
    pub fn new(value: T) -> Self {
        Self {
            inner: parking_lot::RwLock::new(value),
        }
    }
    
    /// Read lock
    pub async fn read(&self) -> parking_lot::RwLockReadGuard<'_, T> {
        self.inner.read()
    }
    
    /// Write lock
    pub async fn write(&self) -> parking_lot::RwLockWriteGuard<'_, T> {
        self.inner.write()
    }
}
