use cycle::prelude::*;
use std::time::Instant;

fn main() {
    println!("🔥 CYCLE Performance Benchmark");
    
    let rt = Runtime::new();
    
    // Benchmark 1: Task spawning speed
    println!("\n📊 Benchmark 1: Task Spawning Speed");
    let start = Instant::now();
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for i in 0..10_000 {
            let handle = spawn(async move { i });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    let elapsed = start.elapsed();
    let tasks_per_sec = 10_000.0 / elapsed.as_secs_f64();
    
    println!("⚡ 10,000 tasks completed in {:?}", elapsed);
    println!("�� Performance: {:.0} tasks/second", tasks_per_sec);
    
    // Benchmark 2: Concurrent execution
    println!("\n📊 Benchmark 2: Concurrent Execution");
    let start = Instant::now();
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let handle = spawn(async move {
                // Simulate some work
                for _ in 0..100 {
                    std::hint::black_box(i * 2);
                }
                i
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    let elapsed = start.elapsed();
    println!("⚡ 1,000 concurrent tasks in {:?}", elapsed);
    
    // Show runtime stats
    let stats = stats();
    println!("\n📈 Runtime Statistics:");
    println!("  Tasks Spawned: {}", stats.tasks_spawned);
    println!("  Tasks Completed: {}", stats.tasks_completed);
    println!("  Active Tasks: {}", stats.active_tasks);
    println!("  Tasks/Second: {:.0}", stats.tasks_per_second());
    println!("  Completion Rate: {:.1}%", stats.completion_rate() * 100.0);
}
