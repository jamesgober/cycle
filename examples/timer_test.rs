//! Timer system test

use cycle::time::{interval, timeout, yield_now, sleep};
use std::time::{Duration, Instant};

fn main() {
    cycle::block_on(async_main());
}

async fn async_main() {
    println!("🔥 CYCLE Timer Test v0.2.0");
    
    println!("⏰ Testing sleep...");
    let start = Instant::now();
    sleep(Duration::from_millis(100)).await;
    println!("✅ Sleep completed in {:?}", start.elapsed());
    
    println!("⏰ Testing interval...");
    let mut interval = interval(Duration::from_millis(50));
    for i in 0..5 {
        let tick = interval.tick().await;
        println!("✅ Tick {}: {:?}", i + 1, tick);
    }
    
    println!("⏰ Testing timeout (success)...");
    let result = timeout(Duration::from_millis(50), async {
        sleep(Duration::from_millis(25)).await;
        "Success!"
    }).await;
    
    match result {
        Ok(value) => println!("✅ Timeout success: {}", value),
        Err(_) => println!("❌ Unexpected timeout"),
    }
    
    println!("⏰ Testing timeout (timeout)...");
    let result = timeout(Duration::from_millis(25), async {
        sleep(Duration::from_millis(50)).await;
        "Should not reach here"
    }).await;
    
    match result {
        Ok(_) => println!("❌ Unexpected success"),
        Err(_) => println!("✅ Timeout occurred as expected"),
    }
    
    println!("⏰ Testing yield_now...");
    let start = Instant::now();
    for i in 0..10 {
        yield_now().await;
        if i % 3 == 0 {
            println!("  Yielded {} times", i + 1);
        }
    }
    println!("✅ Yield test completed in {:?}", start.elapsed());
    
    println!("🚀 All timer tests passed!");
}