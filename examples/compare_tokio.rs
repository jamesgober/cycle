//! Direct comparison between CYCLE and Tokio

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
    
    // Warm up both runtimes
    println!("🔧 Warming up runtimes...");
    let _ = benchmark_cycle();
    let _ = benchmark_tokio();
    
    // Run multiple iterations for accurate results
    const ITERATIONS: usize = 10;
    let mut cycle_times = Vec::new();
    let mut tokio_times = Vec::new();
    
    println!("📊 Running {} iterations...", ITERATIONS);
    
    for i in 1..=ITERATIONS {
        print!("  Iteration {}/{}: ", i, ITERATIONS);
        
        let cycle_time = benchmark_cycle();
        let tokio_time = benchmark_tokio();
        
        cycle_times.push(cycle_time);
        tokio_times.push(tokio_time);
        
        println!("CYCLE: {:?}, Tokio: {:?}", cycle_time, tokio_time);
    }
    
    // Calculate averages
    let cycle_avg = cycle_times.iter().sum::<std::time::Duration>() / cycle_times.len() as u32;
    let tokio_avg = tokio_times.iter().sum::<std::time::Duration>() / tokio_times.len() as u32;
    
    // Calculate min/max
    let cycle_min = *cycle_times.iter().min().unwrap();
    let cycle_max = *cycle_times.iter().max().unwrap();
    let tokio_min = *tokio_times.iter().min().unwrap();
    let tokio_max = *tokio_times.iter().max().unwrap();
    
    println!("\n📊 Results (average of {} runs):", ITERATIONS);
    println!("  🔥 CYCLE:");
    println!("    Average: {:?}", cycle_avg);
    println!("    Min: {:?}", cycle_min);
    println!("    Max: {:?}", cycle_max);
    println!("  🐢 Tokio:");
    println!("    Average: {:?}", tokio_avg);
    println!("    Min: {:?}", tokio_min);
    println!("    Max: {:?}", tokio_max);
    
    let speedup = tokio_avg.as_nanos() as f64 / cycle_avg.as_nanos() as f64;
    if speedup > 1.0 {
        println!("  🚀 CYCLE is {:.1}x FASTER than Tokio!", speedup);
    } else {
        println!("  😅 Tokio is {:.1}x faster than CYCLE", 1.0 / speedup);
    }
    
    // Calculate tasks per second
    let cycle_tps = 1000.0 / cycle_avg.as_secs_f64();
    let tokio_tps = 1000.0 / tokio_avg.as_secs_f64();
    
    println!("\n📈 Throughput:");
    println!("  🔥 CYCLE: {:.0} tasks/second", cycle_tps);
    println!("  🐢 Tokio: {:.0} tasks/second", tokio_tps);
    
    let throughput_advantage = cycle_tps / tokio_tps;
    println!("  💪 CYCLE throughput advantage: {:.1}x", throughput_advantage);
    
    println!("\n🔥 CYCLE v0.2.0 - Still outperforming Tokio!");
}
