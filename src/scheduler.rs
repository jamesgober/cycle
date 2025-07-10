//! Simple task scheduler

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Task type for the scheduler
pub type Task = Box<dyn FnOnce() + Send>;

/// Simple scheduler with round-robin assignment
pub struct Scheduler {
    /// Global task queue
    global: Arc<Mutex<VecDeque<Task>>>,
    
    /// Per-worker local queues
    workers: Vec<Arc<Mutex<VecDeque<Task>>>>,
    
    /// Next worker for round-robin assignment
    next_worker: AtomicUsize,
    
    /// Number of workers
    num_workers: usize,
}

impl Scheduler {
    /// Create new scheduler
    pub fn new(num_workers: usize) -> Self {
        let mut workers = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            workers.push(Arc::new(Mutex::new(VecDeque::new())));
        }
        
        Self {
            global: Arc::new(Mutex::new(VecDeque::new())),
            workers,
            next_worker: AtomicUsize::new(0),
            num_workers,
        }
    }
    
    /// Schedule a task
    pub fn schedule(&self, task: Task) {
        let worker_id = self.next_worker.fetch_add(1, Ordering::Relaxed) % self.num_workers;
        
        // Try local queue first
        if let Ok(mut queue) = self.workers[worker_id].try_lock() {
            queue.push_back(task);
        } else {
            // Fallback to global queue
            self.global.lock().unwrap().push_back(task);
        }
    }
    
    /// Find work for a worker
    pub fn find_work(&self, worker_id: usize) -> Option<Task> {
        // 1. Check local queue first
        if let Ok(mut queue) = self.workers[worker_id].try_lock() {
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }
        
        // 2. Check global queue
        if let Ok(mut queue) = self.global.try_lock() {
            if let Some(task) = queue.pop_front() {
                return Some(task);
            }
        }
        
        // 3. Try stealing from other workers
        self.steal_work(worker_id)
    }
    
    /// Steal work from other workers
    fn steal_work(&self, worker_id: usize) -> Option<Task> {
        for i in 0..self.num_workers {
            if i == worker_id {
                continue;
            }
            
            if let Ok(mut queue) = self.workers[i].try_lock() {
                if let Some(task) = queue.pop_back() {
                    return Some(task);
                }
            }
        }
        
        None
    }
    
    /// Check if there's any work available
    pub fn has_work(&self) -> bool {
        // Check global queue
        if let Ok(queue) = self.global.try_lock() {
            if !queue.is_empty() {
                return true;
            }
        }
        
        // Check worker queues
        for worker in &self.workers {
            if let Ok(queue) = worker.try_lock() {
                if !queue.is_empty() {
                    return true;
                }
            }
        }
        
        false
    }
}
