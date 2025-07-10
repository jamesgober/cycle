//! Runtime builder for custom configuration

use super::Runtime;

/// Builder for configuring a CYCLE runtime
#[derive(Debug)]
pub struct Builder {
    worker_threads: Option<usize>,
    thread_name: String,
    thread_stack_size: Option<usize>,
    enable_all: bool,
}

impl Builder {
    /// Create a new runtime builder
    pub fn new() -> Self {
        Self {
            worker_threads: None,
            thread_name: "cycle-worker".to_string(),
            thread_stack_size: None,
            enable_all: false,
        }
    }
    
    /// Set the number of worker threads
    pub fn worker_threads(mut self, val: usize) -> Self {
        self.worker_threads = Some(val);
        self
    }
    
    /// Set the name for worker threads
    pub fn thread_name<S: Into<String>>(mut self, val: S) -> Self {
        self.thread_name = val.into();
        self
    }
    
    /// Set the stack size for worker threads
    pub fn thread_stack_size(mut self, val: usize) -> Self {
        self.thread_stack_size = Some(val);
        self
    }
    
    /// Enable all available features
    pub fn enable_all(mut self) -> Self {
        self.enable_all = true;
        self
    }
    
    /// Build the runtime
    pub fn build(self) -> std::io::Result<Runtime> {
        // TODO: Implement actual runtime construction
        Ok(Runtime::new())
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}
