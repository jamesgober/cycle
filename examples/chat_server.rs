//! Simple echo chat server - each client gets their own echo

use cycle::prelude::*;
use cycle::{block_on, spawn};
use cycle::io::{AsyncReadExt, AsyncWriteExt};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() -> io::Result<()> {
    block_on(async_main())
}

async fn async_main() -> io::Result<()> {
    println!("🔥 CYCLE Echo Chat Server v0.2.0");
    println!("⚡ Listening on 127.0.0.1:8081");
    println!("📡 Connect with: telnet 127.0.0.1 8081");
    
    let client_counter = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("127.0.0.1:8081").await?;
    
    loop {
        let (stream, addr) = listener.accept().await?;
        let client_id = client_counter.fetch_add(1, Ordering::Relaxed);
        println!("📡 Client {} connected from {}", client_id, addr);
        
        spawn(async move {
            if let Err(e) = handle_client(client_id, stream).await {
                println!("❌ Client {} error: {}", client_id, e);
            }
        });
    }
}

async fn handle_client(client_id: usize, mut stream: TcpStream) -> io::Result<()> {
    // Send welcome message
    let welcome = format!("🔥 Welcome to CYCLE Echo Chat! You are client {}\n", client_id);
    stream.write_all(welcome.as_bytes()).await?;
    
    let mut buffer = vec![0; 1024];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => {
                println!("🔌 Client {} disconnected", client_id);
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                if !message.is_empty() {
                    println!("💬 Client {}: {}", client_id, message);
                    
                    // Echo back with client ID
                    let echo = format!("[Echo from {}]: {}\n", client_id, message);
                    stream.write_all(echo.as_bytes()).await?;
                }
            }
            Err(e) => {
                println!("❌ Read error from client {}: {}", client_id, e);
                break;
            }
        }
    }
    
    println!("🔌 Client {} left", client_id);
    Ok(())
}