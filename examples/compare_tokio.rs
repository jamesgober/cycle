use std::time::Instant;

fn benchmark_cycle() -> std::time::Duration {
    use cycle::prelude::*;
    
    let rt = Runtime::new();
    let start = Instant::now();
    
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let handle = spawn(async move { i * 2 });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    start.elapsed()
}

fn benchmark_tokio() -> std::time::Duration {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let start = Instant::now();
    
    rt.block_on(async {
        let mut handles = Vec::new();
        
        for i in 0..1000 {
            let handle = tokio::spawn(async move { i * 2 });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    });
    
    start.elapsed()
}

fn main() {
    println!("🔥 CYCLE vs Tokio Performance Comparison");
    println!("⚡ Running 1,000 tasks on each runtime...\n");
    
    // Warm up
    let _ = benchmark_cycle();
    let _ = benchmark_tokio();
    
    // Benchmark CYCLE
    let cycle_times: Vec<_> = (0..5).map(|_| benchmark_cycle()).collect();
    let cycle_avg = cycle_times.iter().sum::<std::time::Duration>() / cycle_times.len() as u32;
    
    // Benchmark Tokio
    let tokio_times: Vec<_> = (0..5).map(|_| benchmark_tokio()).collect();
    let tokio_avg = tokio_times.iter().sum::<std::time::Duration>() / tokio_times.len() as u32;
    
    println!("📊 Results (average of 5 runs):");
    println!("  🔥 CYCLE: {:?}", cycle_avg);
    println!("  🐢 Tokio: {:?}", tokio_avg);
    
    let speedup = tokio_avg.as_nanos() as f64 / cycle_avg.as_nanos() as f64;
    if speedup > 1.0 {
        println!("  🚀 CYCLE is {:.1}x FASTER than Tokio!", speedup);
    } else {
        println!("  😅 Tokio is {:.1}x faster than CYCLE", 1.0 / speedup);
    }
}
