//! Simple HTTP client example

use cycle::prelude::*;
use std::io::{self, Write, Read};

fn main() -> io::Result<()> {
    cycle::block_on(async_main())
}

async fn async_main() -> io::Result<()> {
    println!("🔥 CYCLE HTTP Client v0.2.0");
    
    // Connect to a web server
    println!("🌐 Connecting to httpbin.org...");
    let mut stream = TcpStream::connect("httpbin.org:80").await?;
    
    let request = "GET /get HTTP/1.1\r\nHost: httpbin.org\r\nConnection: close\r\n\r\n";
    println!("📤 Sending HTTP request...");
    stream.write_all(request.as_bytes())?;
    
    // Read response
    println!("📥 Reading response...");
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;
    
    // Display response
    let response_str = String::from_utf8_lossy(&response);
    println!("✅ Response received ({} bytes):", response.len());
    println!("---");
    
    // Only show first 500 chars to avoid spam
    if response_str.len() > 500 {
        println!("{}...", &response_str[..500]);
        println!("(truncated {} chars)", response_str.len() - 500);
    } else {
        println!("{}", response_str);
    }
    
    println!("---");
    println!("🚀 HTTP request completed successfully!");
    
    Ok(())
}
