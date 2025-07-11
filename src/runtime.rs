//! High-performance async runtime with real I/O

use crate::scheduler::{Scheduler, Task};
use crate::task::{JoinHandle, TaskControl};
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Re-export reactor for internal use
pub use crate::reactor::REACTOR;

/// High-performance async runtime with I/O integration
pub struct Runtime {
    /// Task scheduler
    scheduler: Arc<Scheduler>,
    
    /// Worker thread handles
    _workers: Vec<thread::JoinHandle<()>>,
    
    /// Shutdown signal
    shutdown: Arc<AtomicBool>,
    
    /// Runtime statistics
    stats: Arc<RuntimeStats>,
    
    /// Runtime start time
    start_time: Instant,
}

/// Runtime performance statistics
#[derive(Debug, Default)]
pub struct RuntimeStats {
    /// Total tasks spawned
    pub tasks_spawned: AtomicU64,
    
    /// Total tasks completed
    pub tasks_completed: AtomicU64,
    
    /// Current active tasks
    pub active_tasks: AtomicU64,
    
    /// I/O operations completed
    pub io_operations: AtomicU64,
    
    /// Timer operations completed  
    pub timer_operations: AtomicU64,
}

impl Runtime {
    /// Create new runtime with optimal worker count
    pub fn new() -> Self {
        let num_workers = num_cpus::get();
        Self::with_workers(num_workers)
    }
    
    /// Create runtime with specific worker count
    pub fn with_workers(num_workers: usize) -> Self {
        let scheduler = Arc::new(Scheduler::new(num_workers));
        let shutdown = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(RuntimeStats::default());
        
        // Initialize reactor
        once_cell::sync::Lazy::force(&REACTOR);
        
        // Start worker threads
        let workers = Self::start_workers(num_workers, scheduler.clone(), shutdown.clone());
        
        Self {
            scheduler,
            _workers: workers,
            shutdown,
            stats,
            start_time: Instant::now(),
        }
    }
    
    /// Start worker threads
    fn start_workers(
        num_workers: usize,
        scheduler: Arc<Scheduler>,
        shutdown: Arc<AtomicBool>,
    ) -> Vec<thread::JoinHandle<()>> {
        let mut workers = Vec::with_capacity(num_workers);
        
        for worker_id in 0..num_workers {
            let scheduler = scheduler.clone();
            let shutdown = shutdown.clone();
            
            let handle = thread::Builder::new()
                .name(format!("cycle-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_main(worker_id, scheduler, shutdown);
                })
                .expect("Failed to spawn worker thread");
            
            workers.push(handle);
        }
        
        workers
    }
    
    /// Worker thread main loop
    fn worker_main(
        worker_id: usize,
        scheduler: Arc<Scheduler>,
        shutdown: Arc<AtomicBool>,
    ) {
        let mut idle_count = 0;
        const MAX_IDLE: usize = 1000;
        
        while !shutdown.load(Ordering::Acquire) {
            if let Some(task) = scheduler.find_work(worker_id) {
                // Execute task
                task();
                idle_count = 0;
            } else {
                // No work available
                idle_count += 1;
                
                if idle_count < MAX_IDLE {
                    thread::yield_now();
                } else {
                    thread::sleep(Duration::from_micros(100));
                    idle_count = 0;
                }
            }
        }
    }
    
    /// Spawn a task on this runtime
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.stats.tasks_spawned.fetch_add(1, Ordering::Relaxed);
        self.stats.active_tasks.fetch_add(1, Ordering::Relaxed);
        
        let control = TaskControl::new();
        let handle = JoinHandle::new(control.clone());
        
        let control_clone = control.clone();
        let stats_clone = self.stats.clone();
        
        // Create task that executes the future
        let task: Task = Box::new(move || {
            // Execute future using futures executor
            let result = futures_executor::block_on(future);
            control_clone.complete(Ok(result));
            
            stats_clone.tasks_completed.fetch_add(1, Ordering::Relaxed);
            stats_clone.active_tasks.fetch_sub(1, Ordering::Relaxed);
        });
        
        self.scheduler.schedule(task);
        handle
    }
    
    /// Block on a future
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.spawn(future);
        
        // Busy wait for completion with yielding
        while !handle.is_finished() {
            thread::yield_now();
        }
        
        handle.try_result().unwrap().expect("Task failed")
    }
    
    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStatsSnapshot {
        RuntimeStatsSnapshot {
            uptime: self.start_time.elapsed(),
            tasks_spawned: self.stats.tasks_spawned.load(Ordering::Relaxed),
            tasks_completed: self.stats.tasks_completed.load(Ordering::Relaxed),
            active_tasks: self.stats.active_tasks.load(Ordering::Relaxed),
            io_operations: self.stats.io_operations.load(Ordering::Relaxed),
            timer_operations: self.stats.timer_operations.load(Ordering::Relaxed),
        }
    }
    
    /// Shutdown the runtime gracefully
    pub fn shutdown(self) {
        self.shutdown.store(true, Ordering::Release);
        
        // Wait for workers to finish
        for worker in self._workers {
            let _ = worker.join();
        }
        
        // Shutdown reactor
        REACTOR.shutdown();
    }
}

/// Snapshot of runtime statistics
#[derive(Debug, Clone)]
pub struct RuntimeStatsSnapshot {
    /// How long the runtime has been running
    pub uptime: Duration,
    /// Total number of tasks spawned
    pub tasks_spawned: u64,
    /// Total number of tasks completed
    pub tasks_completed: u64,
    /// Current number of active tasks
    pub active_tasks: u64,
    /// Total number of I/O operations completed
    pub io_operations: u64,
    /// Total number of timer operations completed
    pub timer_operations: u64,
}

impl RuntimeStatsSnapshot {
    /// Calculate tasks per second
    pub fn tasks_per_second(&self) -> f64 {
        if self.uptime.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.tasks_completed as f64 / self.uptime.as_secs_f64()
    }
    
    /// Calculate completion rate
    pub fn completion_rate(&self) -> f64 {
        if self.tasks_spawned == 0 {
            return 0.0;
        }
        self.tasks_completed as f64 / self.tasks_spawned as f64
    }
    
    /// Calculate I/O operations per second
    pub fn io_ops_per_second(&self) -> f64 {
        if self.uptime.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.io_operations as f64 / self.uptime.as_secs_f64()
    }
    
    /// Calculate timer operations per second
    pub fn timer_ops_per_second(&self) -> f64 {
        if self.uptime.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.timer_operations as f64 / self.uptime.as_secs_f64()
    }
}
