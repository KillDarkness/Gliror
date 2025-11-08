use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest;
use serde::Serialize;
pub mod user_agents;
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use chrono::DateTime;

// Parse schedule time (either timestamp in seconds or "YYYY-MM-DD HH:MM:SS" format)
fn parse_schedule_time(schedule: Option<String>) -> Option<Instant> {
    if let Some(schedule_str) = schedule {
        // Try parsing as timestamp first
        if let Ok(timestamp) = schedule_str.parse::<u64>() {
            return Some(Instant::now() + Duration::from_secs(timestamp));
        } 
        // Try parsing as datetime format
        else if let Ok(dt) = DateTime::parse_from_str(&schedule_str, "%Y-%m-%d %H:%M:%S") {
            let timestamp = dt.timestamp() as u64;
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            if timestamp > current_time {
                return Some(Instant::now() + Duration::from_secs(timestamp - current_time));
            }
        }
    }
    None
}

#[derive(Serialize)]
struct AttackResults {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    success_rate: f64,
    average_rps: f64,
    average_response_time: f64,
    duration_seconds: f64,
    target_url: String,
    method: String,
}

pub async fn perform_attack(
    url: String, 
    duration: u64, 
    method: String, 
    headers: HashMap<String, String>, 
    data: Option<String>,
    proxy: Option<String>,
    concurrent: Option<u32>,
    delay: u64,
    output_file: Option<String>,
    ramp_up: Option<u64>,
    schedule: Option<String>,
    random_ua: bool,
) {
    // Handle scheduling with countdown
    if let Some(scheduled_time) = parse_schedule_time(schedule) {
        let total_wait_duration = scheduled_time.saturating_duration_since(Instant::now());
        if !total_wait_duration.is_zero() {
            // Show initial message that will be updated
            print!("\r{} Scheduled start in {:.1} seconds", "INFO".blue(), total_wait_duration.as_secs_f64());
            std::io::stdout().flush().unwrap();
            
            // Variables that might be used in future enhancements
            let _start_time = Instant::now();
            let _total_duration = total_wait_duration;
            
            // Show a countdown that updates the same line
            let mut current_wait = total_wait_duration;
            while current_wait.as_secs() > 0 {
                let minutes = current_wait.as_secs() / 60;
                let seconds = current_wait.as_secs() % 60;
                
                if minutes > 0 {
                    print!("\r{} Scheduled start in {} minute{} and {} second{}", "INFO".blue(), minutes, if minutes == 1 { "" } else { "s" }, seconds, if seconds == 1 { "" } else { "s" });
                } else {
                    print!("\r{} Scheduled start in {} second{}", "INFO".blue(), seconds, if seconds == 1 { "" } else { "s" });
                }
                
                std::io::stdout().flush().unwrap();
                
                // Sleep for 1 second
                tokio::time::sleep(Duration::from_secs(1)).await;
                
                // Recalculate remaining time
                current_wait = scheduled_time.saturating_duration_since(Instant::now());
            }
            
            // Final message when countdown completes
            print!("\r{} Scheduled start time reached! Duration: {:.1} seconds              ", "INFO".green(), total_wait_duration.as_secs_f64());
            std::io::stdout().flush().unwrap();
            println!(); // New line after the countdown
        }
    }
    
    let requests_sent = Arc::new(AtomicU64::new(0));
    let requests_success = Arc::new(AtomicU64::new(0));
    let requests_error = Arc::new(AtomicU64::new(0));
    let avg_response_time = Arc::new(Mutex::new(Vec::new()));
    let last_slow_warning = Arc::new(Mutex::new(Instant::now()));
    
    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(10));
    
    if let Some(proxy_url) = proxy {
        if let Ok(p) = reqwest::Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(p);
        } else {
            eprintln!("{} Invalid proxy URL: {}", "ERROR".red(), proxy_url);
        }
    }
    
    let client = client_builder.build().expect("Failed to create client");
    
    let target_concurrent = if let Some(concurrent_val) = concurrent {
        concurrent_val as usize
    } else {
        if duration == 0 { 100 } else { 20 }
    };
    
    // Apply ramp-up logic if specified
    let num_concurrent_requests = if let Some(ramp_val) = ramp_up {
        if ramp_val > 0 {
            println!("{} Ramp-up: {} seconds to {} concurrent requests", "INFO".blue(), ramp_val, target_concurrent);
            // For now, just use the target concurrent, but in future we'll ramp up gradually
            target_concurrent
        } else {
            target_concurrent
        }
    } else {
        target_concurrent
    };
    
    let pb = Arc::new(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    
    let start_time = Instant::now();
    
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
        let random_ua_clone = random_ua;
        
        let task = tokio::spawn(async move {
            loop {
                if duration != 0 && start_time.elapsed().as_secs() >= duration {
                    break;
                }
                
                let request_start = Instant::now();
                
                let mut request_builder = match method_clone.as_str() {
                    "GET" => client_clone.get(&url_clone),
                    "POST" => client_clone.post(&url_clone),
                    "PUT" => client_clone.put(&url_clone),
                    "DELETE" => client_clone.delete(&url_clone),
                    "PATCH" => client_clone.patch(&url_clone),
                    "HEAD" => client_clone.head(&url_clone),
                    _ => client_clone.get(&url_clone),
                };
                
                let headers = headers_clone.clone();

                // Add random User-Agent if enabled and not already set
                if random_ua_clone && !headers.keys().any(|k| k.eq_ignore_ascii_case("user-agent")) {
                    request_builder = request_builder.header("User-Agent", user_agents::get_random_user_agent());
                }

                for (key, value) in &headers {
                    request_builder = request_builder.header(key, value);
                }
                
                if let Some(ref payload) = data_clone {
                    request_builder = request_builder.body(payload.clone());
                }
                
                match request_builder.send().await {
                    Ok(_response) => {
                        let elapsed = request_start.elapsed().as_millis() as f64;
                        
                        requests_sent_clone.fetch_add(1, Ordering::SeqCst);
                        requests_success_clone.fetch_add(1, Ordering::SeqCst);
                        
                        {
                            let mut times = avg_response_time_clone.lock().unwrap();
                            times.push(elapsed);
                        }
                        
                        if elapsed > 2000.0 {
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
                
                if delay_clone > 0 {
                    tokio::time::sleep(Duration::from_millis(delay_clone)).await;
                }
                
                if duration == 0 && delay_clone == 0 {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        });
        
        tasks.push(task);
    }
    
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
                
                let avg_time = {
                    let times = avg_response_time_status.lock().unwrap();
                    if !times.is_empty() {
                        times.iter().sum::<f64>() / times.len() as f64
                    } else {
                        0.0
                    }
                };
                
                let elapsed_time = start_time.elapsed().as_secs() as f64;
                let rps = if elapsed_time > 0.0 { sent as f64 / elapsed_time } else { 0.0 };
                
                pb_clone.set_message(format!(
                    "{} Sent: {}, Success: {}, Errors: {}, RPS: {:.1}, Avg: {:.2}ms",
                    "STATUS:".green(),
                    sent,
                    success,
                    error,
                    rps,
                    avg_time
                ));
                
                if error > 0 && sent > 0 {
                    let error_rate = (error as f64 / sent as f64) * 100.0;
                    if error_rate > 10.0 {
                        println!("\n{} High error rate detected: {:.2}% errors", "WARNING".yellow(), error_rate);
                    }
                }
                
                if avg_time > 1500.0 {
                    println!("\n{} Average response time is high: {:.2}ms", "WARNING".yellow(), avg_time);
                }
            }
        }
    });
    
    if duration > 0 {
        tokio::time::sleep(Duration::from_secs(duration)).await;
        
        status_handle.abort();
        
        for task in tasks {
            let _ = task.await;
        }
    } else {
        println!("\n{} Attack started in unlimited mode. Press Ctrl+C to stop.", "INFO".blue());
        
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
    
    pb.finish_and_clear();
    
    let total_sent = requests_sent.load(Ordering::SeqCst);
    let total_success = requests_success.load(Ordering::SeqCst);
    let total_error = requests_error.load(Ordering::SeqCst);
    
    println!("\n{}", "ATTACK COMPLETED".green().bold());
    println!("Total requests sent: {}", total_sent);
    println!("Successful requests: {}", total_success);
    println!("Failed requests: {}", total_error);
    
    let success_rate = if total_sent > 0 {
        (total_success as f64 / total_sent as f64) * 100.0
    } else {
        0.0
    };
    
    if total_sent > 0 {
        println!("Success rate: {:.2}%", success_rate);
        
        let elapsed_time = start_time.elapsed().as_secs() as f64;
        let avg_rps = total_sent as f64 / elapsed_time;
        println!("Average requests per second: {:.1}", avg_rps);
    }
    
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
    
    if let Some(output_path) = output_file {
        let results = AttackResults {
            total_requests: total_sent,
            successful_requests: total_success,
            failed_requests: total_error,
            success_rate,
            average_rps: if total_sent > 0 { total_sent as f64 / start_time.elapsed().as_secs_f64() } else { 0.0 },
            average_response_time: avg_time,
            duration_seconds: start_time.elapsed().as_secs_f64(),
            target_url: url.clone(),
            method: method.clone(),
        };
        
        match serde_json::to_string_pretty(&results) {
            Ok(json) => {
                match std::fs::write(&output_path, json) {
                    Ok(_) => println!("\n{} Results saved to: {}", "INFO".blue(), output_path),
                    Err(e) => eprintln!("\n{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                }
            }
            Err(e) => eprintln!("\n{} Failed to serialize results: {}", "ERROR".red(), e),
        }
    }
}