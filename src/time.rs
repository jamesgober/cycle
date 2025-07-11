//! High-performance timer system with timer wheels

use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use std::thread;
use once_cell::sync::Lazy;

/// Re-export standard time types
pub use std::time::{SystemTime, UNIX_EPOCH};

/// Global timer wheel
static TIMER_WHEEL: Lazy<Arc<TimerWheel>> = Lazy::new(|| {
    let wheel = Arc::new(TimerWheel::new());
    let wheel_clone = wheel.clone();
    
    // Start timer thread
    thread::spawn(move || {
        wheel_clone.run();
    });
    
    wheel
});

/// Timer wheel for efficient timer management
struct TimerWheel {
    timers: Mutex<BinaryHeap<Reverse<Timer>>>,
    next_id: std::sync::atomic::AtomicU64,
}

/// Individual timer
#[derive(Debug)]
struct Timer {
    id: u64,
    deadline: Instant,
    waker: Option<Waker>,
}



impl TimerWheel {
    fn new() -> Self {
        Self {
            timers: Mutex::new(BinaryHeap::new()),
            next_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
    
    fn add_timer(&self, deadline: Instant, waker: Waker) -> u64 {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let timer = Timer {
            id,
            deadline,
            waker: Some(waker),
        };
        
        self.timers.lock().unwrap().push(Reverse(timer));
        id
    }
    
    fn run(&self) {
        loop {
            let now = Instant::now();
            let mut expired_timers = Vec::new();
            
            // Collect expired timers
            {
                let mut timers = self.timers.lock().unwrap();
                while let Some(Reverse(timer)) = timers.peek() {
                    if timer.deadline <= now {
                        if let Some(Reverse(timer)) = timers.pop() {
                            expired_timers.push(timer);
                        }
                    } else {
                        break;
                    }
                }
            }
            
            // Wake expired timers
            for timer in expired_timers {
                if let Some(waker) = timer.waker {
                    waker.wake();
                }
            }
            
            // Sleep for a short time
            thread::sleep(Duration::from_millis(1));
        }
    }
}

/// Sleep for the specified duration
pub async fn sleep(duration: Duration) -> () {
    SleepFuture::new(duration).await
}

/// Sleep until a specific instant
pub async fn sleep_until(deadline: Instant) -> () {
    SleepUntilFuture::new(deadline).await
}

/// Create a timeout future
pub async fn timeout<F>(duration: Duration, future: F) -> Result<F::Output, TimeoutError>
where
    F: Future,
{
    TimeoutFuture::new(duration, future).await
}

/// Sleep future implementation
struct SleepFuture {
    deadline: Instant,
    timer_id: Option<u64>,
}

impl SleepFuture {
    fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            timer_id: None,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let now = Instant::now();
        
        if now >= self.deadline {
            return Poll::Ready(());
        }
        
        if self.timer_id.is_none() {
            let timer_id = TIMER_WHEEL.add_timer(self.deadline, cx.waker().clone());
            self.timer_id = Some(timer_id);
        }
        
        Poll::Pending
    }
}

/// Sleep until future implementation
struct SleepUntilFuture {
    deadline: Instant,
    timer_id: Option<u64>,
}

impl SleepUntilFuture {
    fn new(deadline: Instant) -> Self {
        Self {
            deadline,
            timer_id: None,
        }
    }
}

impl Future for SleepUntilFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let now = Instant::now();
        
        if now >= self.deadline {
            return Poll::Ready(());
        }
        
        if self.timer_id.is_none() {
            let timer_id = TIMER_WHEEL.add_timer(self.deadline, cx.waker().clone());
            self.timer_id = Some(timer_id);
        }
        
        Poll::Pending
    }
}

pin_project_lite::pin_project! {
    /// Timeout future implementation
    struct TimeoutFuture<F> {
        #[pin]
        future: F,
        deadline: Instant,
        timer_id: Option<u64>,
    }
}

impl<F: Future> TimeoutFuture<F> {
    fn new(duration: Duration, future: F) -> Self {
        Self {
            future,
            deadline: Instant::now() + duration,
            timer_id: None,
        }
    }
}

impl<F: Future> Future for TimeoutFuture<F> {
    type Output = Result<F::Output, TimeoutError>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        
        // Check if future completed
        if let Poll::Ready(output) = this.future.poll(cx) {
            return Poll::Ready(Ok(output));
        }
        
        // Check timeout
        let now = Instant::now();
        if now >= *this.deadline {
            return Poll::Ready(Err(TimeoutError));
        }
        
        // Register timer if not already done
        if this.timer_id.is_none() {
            let timer_id = TIMER_WHEEL.add_timer(*this.deadline, cx.waker().clone());
            *this.timer_id = Some(timer_id);
        }
        
        Poll::Pending
    }
}

/// Timeout error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

/// Interval timer for periodic execution
pub struct Interval {
    period: Duration,
    next_tick: Instant,
}

impl Interval {
    /// Create new interval
    pub fn new(period: Duration) -> Self {
        Self {
            period,
            next_tick: Instant::now() + period,
        }
    }
    
    /// Wait for next tick
    pub async fn tick(&mut self) -> Instant {
        let tick_time = self.next_tick;
        sleep_until(tick_time).await;
        self.next_tick += self.period;
        tick_time
    }
    
    /// Get period
    pub fn period(&self) -> Duration {
        self.period
    }
}

/// Create a new interval
pub fn interval(period: Duration) -> Interval {
    Interval::new(period)
}

/// Yield execution to allow other tasks to run
pub async fn yield_now() {
    struct YieldFuture {
        yielded: bool,
    }
    
    impl Future for YieldFuture {
        type Output = ();
        
        fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                Poll::Pending
            }
        }
    }
    
    YieldFuture { yielded: false }.await
}

/// Delay execution for at least the specified duration
pub async fn delay_for(duration: Duration) {
    sleep(duration).await
}

/// Delay execution until the specified instant
pub async fn delay_until(deadline: Instant) {
    sleep_until(deadline).await
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deadline.cmp(&other.deadline)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline && self.id == other.id
    }
}

impl Eq for Timer {}
