//! File operations example

use tokio::fs::{File, write, read_to_string, remove_file};
use tokio::io::copy;
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("🔥 CYCLE File Operations v0.2.0");
    
    // Create test file
    let test_content = "Hello from CYCLE v0.2.0!\nThis is a test file.\nNow with real async I/O!\n";
    println!("📝 Writing test file...");
    write("test_input.txt", test_content).await?;
    
    // Read file back
    println!("📖 Reading test file...");
    let read_content = read_to_string("test_input.txt").await?;
    println!("✅ Read content ({} bytes):", read_content.len());
    for line in read_content.lines() {
        println!("  > {}", line);
    }
    
    // Copy file using async copy
    println!("📋 Copying file...");
    let mut source = File::open("test_input.txt").await?;
    let mut dest = File::create("test_output.txt").await?;
    let bytes_copied = copy(&mut source, &mut dest).await?;
    println!("✅ Copied {} bytes", bytes_copied);
    
    // Verify copy
    let copied_content = read_to_string("test_output.txt").await?;
    if copied_content == test_content {
        println!("✅ File copy successful!");
    } else {
        println!("❌ File copy failed!");
    }
    
    // Test file metadata
    println!("📊 File metadata:");
    let metadata = File::open("test_input.txt").await?.metadata().await?;
    println!("  Size: {} bytes", metadata.len());
    println!("  Modified: {:?}", metadata.modified());
    // Clean up
    // Clean up
    println!("🧹 Cleaning up...");
    remove_file("test_input.txt").await?;
    remove_file("test_output.txt").await?;
    println!("🚀 File operations test completed!");
    Ok(())
}
