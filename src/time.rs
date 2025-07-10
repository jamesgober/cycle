//! Time utilities and async sleep

use std::time::{Duration, Instant};

/// Sleep for the specified duration
pub async fn sleep(duration: Duration) {
    let start = Instant::now();
    
    while start.elapsed() < duration {
        yield_now().await;
        std::thread::sleep(Duration::from_micros(100));
    }
}

/// Yield execution to other tasks
pub async fn yield_now() {
    struct YieldFuture {
        yielded: bool,
    }
    
    impl std::future::Future for YieldFuture {
        type Output = ();
        
        fn poll(mut self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<()> {
            if self.yielded {
                std::task::Poll::Ready(())
            } else {
                self.yielded = true;
                std::task::Poll::Pending
            }
        }
    }
    
    YieldFuture { yielded: false }.await
}
