//! Echo server example using CYCLE networking

use cycle::prelude::*;
use cycle::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

fn main() -> io::Result<()> {
    cycle::block_on(async_main())
}

async fn async_main() -> io::Result<()> {
    println!("🔥 CYCLE Echo Server v0.2.0");
    println!("⚡ Listening on 127.0.0.1:8080");
    println!("📡 Connect with: telnet 127.0.0.1 8080");
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        let (mut stream, addr) = listener.accept().await?;
        println!("📡 New connection from {}", addr);
        
        spawn(async move {
            let mut buffer = vec![0; 1024];
            
            loop {
                match stream.read(&mut buffer).await {
                    Ok(0) => {
                        println!("🔌 Connection closed: {}", addr);
                        break;
                    }
                    Ok(n) => {
                        println!("📥 Received {} bytes from {}", n, addr);
                        
                        // Echo back with a prefix
                        let echo_msg = format!("ECHO: {}", 
                            String::from_utf8_lossy(&buffer[..n]).trim());
                        
                        if let Err(e) = stream.write_all(echo_msg.as_bytes()).await {
                            println!("❌ Write error: {}", e);
                            break;
                        }
                        
                        println!("📤 Echoed {} bytes to {}", echo_msg.len(), addr);
                    }
                    Err(e) => {
                        println!("❌ Read error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
