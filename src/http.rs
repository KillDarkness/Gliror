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
use chrono::{DateTime, Utc, Local};
use serde_xml_rs;
use csv::Writer;
use std::path::Path;

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

// Helper function to determine the output format from the file extension
fn get_output_format(output_path: &str) -> &'static str {
    let path = std::path::Path::new(output_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => "json",
        Some("xml") => "xml",
        Some("yaml") | Some("yml") => "yaml",
        Some("csv") => "csv",
        Some("toml") => "toml",
        _ => "json", // default to json
    }
}

// Helper function to calculate percentiles from response times
fn calculate_percentiles(times: &[f64]) -> (f64, f64, f64) {
    if times.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    
    let mut sorted_times = times.to_vec();
    sorted_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let len = sorted_times.len();
    
    // Median (50th percentile)
    let median = if len % 2 == 0 {
        (sorted_times[len / 2 - 1] + sorted_times[len / 2]) / 2.0
    } else {
        sorted_times[len / 2]
    };
    
    // 95th percentile
    let p95_idx = ((0.95 * len as f64) as usize).min(len - 1);
    let p95 = sorted_times[p95_idx];
    
    // 99th percentile
    let p99_idx = ((0.99 * len as f64) as usize).min(len - 1);
    let p99 = sorted_times[p99_idx];
    
    (median, p95, p99)
}

#[derive(Serialize)]
struct RequestStats {
    sent: u64,
    success: u64,
    error: u64,
    response_time_ms: f64,
}

#[derive(Serialize)]
struct DetailedAttackResults {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    success_rate: f64,
    average_rps: f64,
    min_response_time: f64,
    max_response_time: f64,
    average_response_time: f64,
    median_response_time: f64,
    p95_response_time: f64,
    p99_response_time: f64,
    duration_seconds: f64,
    start_time: String,  // ISO 8601 timestamp
    end_time: String,    // ISO 8601 timestamp
    target_url: String,
    method: String,
    headers: HashMap<String, String>,
    data_size_bytes: usize,
    concurrent_requests: usize,
    delay_ms: u64,
    total_bytes_transferred: u64,
    requests_per_second_timeline: Vec<RequestStats>, // For time-series analysis
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
        
        // Calculate detailed statistics
        let avg_response_times = avg_response_time.lock().unwrap();
        let response_times: Vec<f64> = avg_response_times.clone();
        let (median_response_time, p95_response_time, p99_response_time) = calculate_percentiles(&response_times);

        let min_response_time = if !response_times.is_empty() {
            response_times.iter().fold(f64::INFINITY, |a, &b| a.min(b))
        } else {
            0.0
        };

        let max_response_time = if !response_times.is_empty() {
            response_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        } else {
            0.0
        };

        let start_time_local = chrono::Local::now();
        let end_time_local = chrono::Local::now();

        // Calculate total bytes transferred (approximation based on request/response sizes)
        let data_size_bytes = data.as_ref().map(|d| d.len()).unwrap_or(0);
        let total_bytes_transferred = total_sent * (data_size_bytes as u64); // Approximation

        // Use target_concurrent for concurrent_requests
        let concurrent_requests = if let Some(concurrent_val) = concurrent {
            concurrent_val as usize
        } else {
            if duration == 0 { 100 } else { 20 }
        };

        let detailed_results = DetailedAttackResults {
            total_requests: total_sent,
            successful_requests: total_success,
            failed_requests: total_error,
            success_rate,
            average_rps: if total_sent > 0 { total_sent as f64 / start_time.elapsed().as_secs_f64() } else { 0.0 },
            min_response_time,
            max_response_time,
            average_response_time: avg_time,
            median_response_time,
            p95_response_time,
            p99_response_time,
            duration_seconds: start_time.elapsed().as_secs_f64(),
            start_time: start_time_local.to_rfc3339(),
            end_time: end_time_local.to_rfc3339(),
            target_url: url.clone(),
            method: method.clone(),
            headers: headers.clone(),
            data_size_bytes,
            concurrent_requests,
            delay_ms: delay,
            total_bytes_transferred,
            requests_per_second_timeline: vec![], // We could implement this for time-series data if needed
        };

        // Determine output format based on file extension
        let format = get_output_format(&output_path);

        match format {
            "json" => {
                match serde_json::to_string_pretty(&detailed_results) {
                    Ok(json) => {
                        match std::fs::write(&output_path, json) {
                            Ok(_) => println!("\n{} Results saved to: {} (JSON format)", "INFO".blue(), output_path),
                            Err(e) => eprintln!("\n{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                        }
                    }
                    Err(e) => eprintln!("\n{} Failed to serialize results to JSON: {}", "ERROR".red(), e),
                }
            }
            "xml" => {
                match serde_xml_rs::to_string(&detailed_results) {
                    Ok(xml) => {
                        match std::fs::write(&output_path, xml) {
                            Ok(_) => println!("\n{} Results saved to: {} (XML format)", "INFO".blue(), output_path),
                            Err(e) => eprintln!("\n{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                        }
                    }
                    Err(e) => eprintln!("\n{} Failed to serialize results to XML: {}", "ERROR".red(), e),
                }
            }
            "yaml" => {
                match serde_yaml::to_string(&detailed_results) {
                    Ok(yaml) => {
                        match std::fs::write(&output_path, yaml) {
                            Ok(_) => println!("\n{} Results saved to: {} (YAML format)", "INFO".blue(), output_path),
                            Err(e) => eprintln!("\n{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                        }
                    }
                    Err(e) => eprintln!("\n{} Failed to serialize results to YAML: {}", "ERROR".red(), e),
                }
            }
            "csv" => {
                // For CSV, we can create a simplified version with just key metrics
                let mut wtr = csv::Writer::from_path(&output_path).unwrap();
                
                // Write headers
                wtr.write_record(&["total_requests", "successful_requests", "failed_requests", 
                                  "success_rate", "average_rps", "min_response_time", 
                                  "max_response_time", "average_response_time", 
                                  "median_response_time", "p95_response_time", 
                                  "p99_response_time", "duration_seconds", 
                                  "target_url", "method"])
                    .unwrap();

                // Write data row
                wtr.write_record(&[
                    total_sent.to_string(),
                    total_success.to_string(),
                    total_error.to_string(),
                    success_rate.to_string(),
                    (if total_sent > 0 { total_sent as f64 / start_time.elapsed().as_secs_f64() } else { 0.0 }).to_string(),
                    min_response_time.to_string(),
                    max_response_time.to_string(),
                    avg_time.to_string(),
                    median_response_time.to_string(),
                    p95_response_time.to_string(),
                    p99_response_time.to_string(),
                    start_time.elapsed().as_secs_f64().to_string(),
                    url.clone(),
                    method.clone(),
                ]).unwrap();
                
                wtr.flush().unwrap();
                println!("\n{} Results saved to: {} (CSV format)", "INFO".blue(), output_path);
            }
            "toml" => {
                match toml::to_string(&detailed_results) {
                    Ok(toml_str) => {
                        match std::fs::write(&output_path, toml_str) {
                            Ok(_) => println!("\n{} Results saved to: {} (TOML format)", "INFO".blue(), output_path),
                            Err(e) => eprintln!("\n{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                        }
                    }
                    Err(e) => eprintln!("\n{} Failed to serialize results to TOML: {}", "ERROR".red(), e),
                }
            }
            _ => {
                eprintln!("\n{} Unsupported file format for: {}", "ERROR".red(), output_path);
            }
        }
    }
}