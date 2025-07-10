//! High-performance async runtime

use crate::scheduler::{Scheduler, Task};
use crate::task::{JoinHandle, TaskControl};
use std::future::Future;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// High-performance async runtime
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
        const MAX_IDLE: usize = 100;
        
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
                    thread::sleep(Duration::from_micros(10));
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
        
        // Create simple executor task
        let task: Task = Box::new(move || {
            // Execute future using simple polling
            let result = futures::executor::block_on(future);
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
        
        // Busy wait for completion
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
        }
    }
}

/// Snapshot of runtime statistics
#[derive(Debug, Clone)]
pub struct RuntimeStatsSnapshot {
    pub uptime: Duration,
    pub tasks_spawned: u64,
    pub tasks_completed: u64,
    pub active_tasks: u64,
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
}
