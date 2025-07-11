use cycle::prelude::*;
use std::time::Instant;

fn main() {
    println!("🔥 CYCLE Performance Benchmark");
    
    let rt = Runtime::new();
    
    // Benchmark task spawning
    let start = Instant::now();
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let handle = spawn(async move { i });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    let elapsed = start.elapsed();
    let tasks_per_sec = 1000.0 / elapsed.as_secs_f64();
    
    println!("⚡ 1,000 tasks completed in {:?}", elapsed);
    println!("🚀 Performance: {:.0} tasks/second", tasks_per_sec);
}