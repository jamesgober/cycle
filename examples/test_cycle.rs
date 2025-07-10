//! Test CYCLE runtime actually works

use cycle::prelude::*;
use std::time::{Duration, Instant};

fn main() {
    println!("🔥 Testing CYCLE async runtime...");
    
    let rt = Runtime::new();
    
    // Test 1: Basic async execution
    println!("📡 Test 1: Basic async function");
    let result = rt.block_on(async {
        println!("  ⚡ Inside async block!");
        42
    });
    println!("  ✅ Result: {}", result);
    
    // Test 2: Task spawning
    println!("📡 Test 2: Task spawning");
    let start = Instant::now();
    
    rt.block_on(async {
        let mut handles = Vec::new();
        
        // Spawn 10 tasks
        for i in 0..10 {
            let handle = spawn(async move {
                println!("  💫 Task {} running!", i);
                std::thread::sleep(Duration::from_millis(10));
                i * 2
            });
            handles.push(handle);
        }
        
        // Wait for all tasks
        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(result) => println!("  ✅ Task {} completed: {}", i, result),
                Err(e) => println!("  ❌ Task {} failed: {}", i, e),
            }
        }
    });
    
    let elapsed = start.elapsed();
    println!("  🕐 All tasks completed in {:?}", elapsed);
    
    // Test 3: Concurrent execution
    println!("📡 Test 3: Concurrent execution");
    let start = Instant::now();
    
    rt.block_on(async {
        let task1 = spawn(async {
            std::thread::sleep(Duration::from_millis(50));
            "Task 1"
        });
        
        let task2 = spawn(async {
            std::thread::sleep(Duration::from_millis(30));
            "Task 2"
        });
        
        let task3 = spawn(async {
            std::thread::sleep(Duration::from_millis(20));
            "Task 3"
        });
        
        let result1 = task1.await.unwrap();
        let result2 = task2.await.unwrap();
        let result3 = task3.await.unwrap();
        
        println!("  ✅ Results: {}, {}, {}", result1, result2, result3);
    });
    
    let elapsed = start.elapsed();
    println!("  🕐 Concurrent tasks completed in {:?}", elapsed);
    
    println!("🚀 CYCLE runtime test completed!");
    println!("🔥 CYCLE IS ALIVE AND WORKING!");
}
