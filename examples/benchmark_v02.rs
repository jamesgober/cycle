//! CYCLE v0.2.0 comprehensive benchmark

use std::time::{Duration, Instant};
use cycle::{spawn, stats};
use tokio::task::{spawn_blocking, yield_now};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("🔥 CYCLE v0.2.0 Performance Benchmark");
    println!("⚡ Testing all core features...\n");
    
    println!("📊 Benchmark 1: Task Spawning");
    let start = Instant::now();
    let mut handles = Vec::new();
    for i in 0..10_000 {
        let handle = spawn(async move { i * 2 });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let tasks_per_sec = 10_000.0 / elapsed.as_secs_f64();
    println!("⚡ 10,000 tasks in {:?}", elapsed);
    println!("🚀 Performance: {:.0} tasks/second\n", tasks_per_sec);
    
    // Concurrent task benchmark
    println!("📊 Benchmark 2: Concurrent Tasks");
    let start = Instant::now();
    
    let mut concurrent_handles = Vec::new();
    for i in 0..1000 {
        let handle = spawn(async move {
            // Simulate work
            for j in 0..i % 100 {
                yield_now().await;
                let _ = j * 2;
            }
            i
        });
        concurrent_handles.push(handle);
    }
    
    for handle in concurrent_handles {
        handle.await.unwrap();
    }
    
    let concurrent_elapsed = start.elapsed();
    println!("⚡ 1,000 concurrent tasks in {:?}", concurrent_elapsed);
    println!("🚀 Concurrent ops/sec: {:.0}\n", 1000.0 / concurrent_elapsed.as_secs_f64());
    
    // Timer benchmark
    #[cfg(feature = "time")]
    {
        println!("📊 Benchmark 3: Timer Operations");
        let start = Instant::now();
        
        let mut timer_handles = Vec::new();
        for i in 0..1000 {
            let handle = spawn(async move {
                sleep(Duration::from_millis(i % 10)).await;
                i
            });
            timer_handles.push(handle);
        }
        
        for handle in timer_handles {
            handle.await.unwrap();
        }
        
        let timer_elapsed = start.elapsed();
        println!("⚡ 1,000 timer operations in {:?}", timer_elapsed);
        println!("🚀 Timer ops/sec: {:.0}\n", 1000.0 / timer_elapsed.as_secs_f64());
    }
    
    // Blocking operations benchmark
    println!("📊 Benchmark 4: Blocking Operations");
    let start = Instant::now();
    
    let mut blocking_handles = Vec::new();
    for i in 0..1000 {
        let handle = spawn_blocking(move || {
            // Simulate CPU work
            let mut sum = 0;
            for j in 0..i {
                sum += j;
            }
            sum
        });
        blocking_handles.push(handle);
    }
    
    for handle in blocking_handles {
        handle.await.unwrap();
    }
    
    let blocking_elapsed = start.elapsed();
    println!("⚡ 1,000 blocking tasks in {:?}", blocking_elapsed);
    println!("🚀 Blocking ops/sec: {:.0}\n", 1000.0 / blocking_elapsed.as_secs_f64());
    
    // Overall statistics
    let stats = stats();
    println!("📈 Overall Runtime Statistics:");
    println!("  Uptime: {:?}", stats.uptime);
    println!("  Tasks Spawned: {}", stats.tasks_spawned);
    println!("  Tasks Completed: {}", stats.tasks_completed);
    println!("  Active Tasks: {}", stats.active_tasks);
    println!("  Tasks/Second: {:.0}", stats.tasks_per_second());
    println!("  Completion Rate: {:.1}%", stats.completion_rate() * 100.0);
    
    #[cfg(feature = "net")]
    println!("  I/O Operations: {}", stats.io_operations);
    
    #[cfg(feature = "time")]
    println!("  Timer Operations: {}", stats.timer_operations);
    
    println!("\n🔥 CYCLE v0.2.0 - Core features complete!");
    println!("⚡ Ready for performance optimizations in v0.3.0+");
}
