mod cli;
mod http;
mod display;
mod stats;
mod utils;
mod cluster;

use clap::Parser;
use cli::Args;
use colored::Colorize;
use display::print_cyan_ascii;
use http::perform_attack;
use cli::parse_headers;
use cluster::{ClusterCoordinator, ClusterConfig, WorkerState, WorkerStatus, WorkerProgress};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("GLIROR {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    
    print_cyan_ascii();
    
    // Handle cluster mode
    if args.cluster_mode {
        println!("{} Cluster mode enabled", "INFO".blue());
        
        let args_clone = args.clone();
        let coordinator_addr = if let Some(addr) = args_clone.coordinator_addr {
            // If coordinator address is provided, use it as is
            addr
        } else {
            // Otherwise, build the address with the specified port (default 8080)
            let port = args_clone.port.unwrap_or(8080);
            format!("http://localhost:{}", port)
        };
        let total_workers = args_clone.total_workers.unwrap_or(2);
        let worker_id = args_clone.worker_id.unwrap_or_else(|| {
            use uuid::Uuid;
            Uuid::new_v4().to_string()
        });
        
        let _config = ClusterConfig::new(total_workers, coordinator_addr);
        
        if args.role.as_deref() == Some("master") {
            println!("{} Running in master mode", "INFO".blue());
            run_master_node(args).await;
        } else if args.role.as_deref() == Some("worker") {
            println!("{} Running in worker mode with ID: {}", "INFO".blue(), worker_id);
            run_worker_node(args).await;
        } else {
            // Default to standalone mode
            handle_standalone_attack(args).await;
        }
    } else {
        // Default to standalone mode
        handle_standalone_attack(args).await;
    }
}

async fn handle_standalone_attack(args: Args) {
    let target_url = if let Some(url) = args.url {
        url
    } else {
        utils::prompt_for_url()
    };
    
    let duration = if args.time != 0 {
        args.time
    } else {
        utils::prompt_for_time()
    };
    
    let headers = parse_headers(&args.header);
    
    println!("\n{} Starting attack on: {}", "INFO".blue(), target_url);
    println!("{} Method: {}", "INFO".blue(), args.method);
    println!("{} Duration: {} seconds", "INFO".blue(), if duration == 0 { "Unlimited".to_string() } else { duration.to_string() });
    println!("{} Concurrent requests: {}", "INFO".blue(), 
             args.concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }));
    if let Some(ref proxy) = args.proxy {
        println!("{} Proxy: {}", "INFO".blue(), proxy);
    }
    
    perform_attack(target_url, duration, args.method, headers, args.data, args.proxy, args.concurrent, args.delay, args.output, args.ramp_up, args.schedule).await;
}

use cluster::server::run_master_server;
use std::sync::Arc;

async fn run_master_node(args: Args) {
    let args_clone = args.clone();
    let coordinator_addr = if let Some(addr) = args_clone.coordinator_addr {
        // If coordinator address is provided, use it as is
        addr
    } else {
        // Otherwise, build the address with the specified port (default 8080)
        let port = args_clone.port.unwrap_or(8080);
        format!("http://localhost:{}", port)
    };
    
    let config = ClusterConfig::new(
        args_clone.total_workers.unwrap_or(2),
        coordinator_addr
    );

    let coordinator = Arc::new(ClusterCoordinator::new(config));

    // Determine port to use
    let port = args.port.unwrap_or(8080);

    // Run server in a separate thread  
    let server_coordinator = coordinator.clone();
    tokio::spawn(async move {
        run_master_server(server_coordinator, port).await;
    });

    println!("{} Master server started. Listening on http://127.0.0.1:{}", "INFO".blue(), port);

    // Wait for all workers to connect
    let total_workers = coordinator.config.total_workers;
    println!("{} Waiting for {} workers to connect...", "INFO".blue(), total_workers);
    
    // Wait for all workers to connect
    loop {
        let connected_workers = coordinator.workers.read().await.len();
        if connected_workers >= total_workers {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Check if time was provided (different from default of 30)
    let time_provided = args.time != 30; // Default is 30, so if it's different, it was provided
    
    // Prompt for attack details after all workers connected (only if not provided in args)
    let target_url = if let Some(url) = args.url {
        url
    } else {
        utils::prompt_for_url()
    };
    let duration = if time_provided {
        args.time  // Use provided time
    } else {
        utils::prompt_for_time()  // Prompt for time if not provided
    };

    // Set default concurrent requests if not specified, higher for cluster mode
    let effective_concurrent = args.concurrent.unwrap_or_else(|| {
        if duration == 0 { 100 } else { 50 }  // Higher default for cluster to improve performance
    });

    let attack_command = cluster::AttackCommand {
        url: target_url,
        time: duration,
        method: args.method.clone(),
        header: args.header.clone(),
        data: args.data.clone(),
        proxy: args.proxy.clone(),
        concurrent: Some(effective_concurrent),
        delay: args.delay,
        output: args.output.clone(),
        ramp_up: args.ramp_up,
        schedule: args.schedule.clone(),
    };

    // Store the attack command for workers to pick up
    {
        let mut command_lock = coordinator.attack_command.write().await;
        *command_lock = Some(attack_command);
    }

    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    let attack_start_time = std::time::Instant::now();

    loop {
        let progress = coordinator.get_total_progress().await;
        
        let elapsed_secs = attack_start_time.elapsed().as_secs_f64();
        let rps = if elapsed_secs > 0.0 { progress.requests_sent as f64 / elapsed_secs } else { 0.0 };

        pb.set_message(format!(
            "{} Sent: {}, Success: {}, Errors: {}, RPS: {:.1}, Avg: {:.2}ms",
            "STATUS:".green(),
            progress.requests_sent,
            progress.requests_success,
            progress.requests_error,
            rps,
            progress.avg_response_time
        ));

        let current_attack_command = coordinator.attack_command.read().await;
        if let Some(cmd) = current_attack_command.as_ref() {
            if cmd.time > 0 && attack_start_time.elapsed().as_secs() >= cmd.time {
                break;
            }
        } else {
            // This case should ideally not be reached if the command is set after workers connect.
            // However, as a safeguard, if for some reason the command is not present, we can break
            // or handle it based on desired behavior (e.g., continue indefinitely or log an error).
            // For now, we'll break to prevent an infinite loop if the command somehow disappears.
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    pb.finish_and_clear();
    println!("{} Attack duration finished.", "INFO".blue());
}

use std::time::{Duration, Instant};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

async fn run_worker_node(args: Args) {
    let args_clone = args.clone();
    let worker_id = args_clone.worker_id.unwrap_or_else(|| {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    });
    let coordinator_addr = if let Some(addr) = args_clone.coordinator_addr {
        // If coordinator address is provided, use it as is
        addr
    } else {
        // Otherwise, build the address with the specified port (default 8080)
        let port = args_clone.port.unwrap_or(8080);
        format!("http://localhost:{}", port)
    };

    let initial_state = WorkerState {
        id: worker_id.clone(),
        status: WorkerStatus::Initializing,
        progress: WorkerProgress {
            requests_sent: 0,
            requests_success: 0,
            requests_error: 0,
            avg_response_time: 0.0,
            current_rps: 0.0,
        },
        last_seen: None,
    };

    let client = reqwest::Client::new();
    loop {
        println!("{} Attempting to register worker {} with coordinator at {}", "INFO".blue(), worker_id, coordinator_addr);
        let res = client.post(format!("{}/workers", coordinator_addr))
            .json(&initial_state)
            .send()
            .await;

        match res {
            Ok(_) => {
                println!("{} Worker {} successfully registered. Polling for tasks...", "INFO".blue(), worker_id);
                break; // Sai do loop de registro uma vez que o registro é bem-sucedido
            }
            Err(e) => {
                eprintln!("{} Failed to connect to coordinator: {}. Retrying in 5 seconds...", "ERROR".red(), e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
    
    loop {
        let task_res = client.get(format!("{}/workers/{}/task", coordinator_addr, worker_id))
            .send()
            .await;

        match task_res {
            Ok(response) => {
                let response_text = response.text().await.unwrap_or_else(|_| "null".to_string());
                
                match serde_json::from_str::<Option<cluster::AttackCommand>>(&response_text) {
                    Ok(Some(task)) => {
                        println!("{} Task received by worker {}. Starting attack on: {} with {} concurrent requests.", "INFO".blue(), worker_id, task.url, task.concurrent.unwrap_or(10));
                        
                        // Update worker status to Working
                        let _ = client.post(format!("{}/workers/{}/status", coordinator_addr, worker_id))
                            .json(&WorkerStatus::Working)
                            .send()
                            .await;
                        
                        let requests_sent = Arc::new(AtomicU64::new(0));
                        let requests_success = Arc::new(AtomicU64::new(0));
                        let requests_error = Arc::new(AtomicU64::new(0));
                        let avg_response_time_values = Arc::new(Mutex::new(Vec::new()));
                        
                        // Status display for worker
                        use indicatif::{ProgressBar, ProgressStyle};
                        let worker_pb = ProgressBar::new_spinner();
                        worker_pb.set_style(
                            ProgressStyle::default_spinner()
                                .template(&format!("{{spinner:.green}} [{}] {{msg}}", worker_id))
                                .unwrap(),
                        );

                        // Reporter Task for worker stats
                        let reporter_client = client.clone();
                        let reporter_coordinator_addr = coordinator_addr.clone();
                        let reporter_worker_id = worker_id.clone();
                        let sent_clone = requests_sent.clone();
                        let success_clone = requests_success.clone();
                        let error_clone = requests_error.clone();
                        let response_times_clone = avg_response_time_values.clone();
                        let pb_clone = worker_pb.clone();
                        let mut last_sent = 0;
                        let mut last_report_time = Instant::now();
                        tokio::spawn(async move {
                            loop {
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                let current_sent = sent_clone.load(Ordering::Relaxed);
                                let elapsed_since_last_report = last_report_time.elapsed().as_secs_f64();
                                let current_rps = if elapsed_since_last_report > 0.0 {
                                    (current_sent - last_sent) as f64 / elapsed_since_last_report
                                } else {
                                    0.0
                                };
                                last_sent = current_sent;
                                last_report_time = Instant::now();
                                
                                let avg_time = {
                                    let mut times = response_times_clone.lock().unwrap();
                                    let avg = if !times.is_empty() {
                                        times.iter().sum::<f64>() / times.len() as f64
                                    } else {
                                        0.0
                                    };
                                    times.clear(); // Clear for responsive average
                                    avg
                                };
                                
                                // Update progress bar on worker side (overwrites previous line)
                                pb_clone.set_message(format!(
                                    "{} Sent: {}, Success: {}, Errors: {}, RPS: {:.1}, Avg: {:.2}ms",
                                    "STATUS:".green(),
                                    current_sent,
                                    success_clone.load(Ordering::Relaxed),
                                    error_clone.load(Ordering::Relaxed),
                                    current_rps,
                                    avg_time
                                ));
                                
                                let progress = WorkerProgress {
                                    requests_sent: current_sent,
                                    requests_success: success_clone.load(Ordering::Relaxed),
                                    requests_error: error_clone.load(Ordering::Relaxed),
                                    avg_response_time: avg_time,
                                    current_rps,
                                };
                                // Check if the coordinator is still reachable before sending progress
                                if let Err(_) = reporter_client.post(format!("{}/workers/{}/progress", reporter_coordinator_addr, reporter_worker_id))
                                    .json(&progress)
                                    .send()
                                    .await {
                                    // If we can't reach the coordinator, continue trying 
                                    // We don't break here, just continue with the loop
                                }
                            }
                        });
                        
                        // Attacker Tasks
                        let mut tasks = Vec::new();
                        let attack_start_time = std::time::Instant::now();
                        for _ in 0..task.concurrent.unwrap_or(10) {
                            let client_clone = client.clone();
                            let url_clone = task.url.clone();
                            let method_clone = task.method.clone();
                            let headers = parse_headers(&task.header);
                            let data_clone = task.data.clone();
                            let sent_clone = requests_sent.clone();
                            let success_clone = requests_success.clone();
                            let error_clone = requests_error.clone();
                            let response_times_clone = avg_response_time_values.clone();
                            
                            let task_handle = tokio::spawn(async move {
                                loop {
                                    if task.time > 0 && attack_start_time.elapsed().as_secs() >= task.time {
                                        break;
                                    }
                                    let request_start = std::time::Instant::now();
                                    let mut request_builder = match method_clone.as_str() {
                                        "GET" => client_clone.get(&url_clone),
                                        "POST" => client_clone.post(&url_clone),
                                        _ => client_clone.get(&url_clone),
                                    };
                                    for (key, value) in &headers {
                                        request_builder = request_builder.header(key, value);
                                    }
                                    if let Some(ref payload) = data_clone {
                                        request_builder = request_builder.body(payload.clone());
                                    }
                                    
                                    match request_builder.send().await {
                                        Ok(_) => {
                                            let elapsed = request_start.elapsed().as_millis() as f64;
                                            success_clone.fetch_add(1, Ordering::SeqCst);
                                            response_times_clone.lock().unwrap().push(elapsed);
                                        }
                                        Err(_) => {
                                            error_clone.fetch_add(1, Ordering::SeqCst);
                                        }
                                    }
                                    sent_clone.fetch_add(1, Ordering::SeqCst);
                                    
                                    if task.delay > 0 {
                                        tokio::time::sleep(Duration::from_millis(task.delay)).await;
                                    }
                                }
                            });
                            tasks.push(task_handle);
                        }
                        
                        // Wait for all tasks to complete
                        for handle in tasks {
                            let _ = handle.await;
                        }
                        
                        // Update worker status to Completed
                        let _ = client.post(format!("{}/workers/{}/status", coordinator_addr, worker_id))
                            .json(&WorkerStatus::Completed)
                            .send()
                            .await;
                        
                        println!("{} Worker {} attack finished. Waiting for new tasks...", "INFO".blue(), worker_id);
                        
                        // Continue polling for new tasks instead of exiting
                        continue;
                    }
                    Ok(None) => {
                        // Worker is still waiting for a task, continue polling
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Err(e) => {
                        eprintln!("{} Failed to parse task response: {}", "ERROR".red(), e);
                        eprintln!("{} Raw response was: {}", "ERROR".red(), response_text);
                        // Wait a bit before trying again
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Lost connection to coordinator: {}. Attempting to re-register...", "ERROR".red(), e);
                
                // Wait a bit before trying to reconnect
                tokio::time::sleep(Duration::from_secs(3)).await;
                
                // Try to register with the coordinator again
                let initial_state = WorkerState {
                    id: worker_id.clone(),
                    status: WorkerStatus::Initializing,
                    progress: WorkerProgress {
                        requests_sent: 0,
                        requests_success: 0,
                        requests_error: 0,
                        avg_response_time: 0.0,
                        current_rps: 0.0,
                    },
                    last_seen: None,
                };

                loop {
                    match client.post(format!("{}/workers", coordinator_addr))
                        .json(&initial_state)
                        .send()
                        .await {
                        Ok(_) => {
                            println!("{} Worker {} successfully re-registered with coordinator.", "INFO".blue(), worker_id);
                            break; // Successfully re-registered, exit the reconnection loop
                        }
                        Err(e) => {
                            eprintln!("{} Reconnection attempt failed: {}. Retrying in 3 seconds...", "ERROR".red(), e);
                            tokio::time::sleep(Duration::from_secs(3)).await;
                            continue;
                        }
                    }
                }
                
                continue; // Continue the main loop to start polling tasks again
            }
        }
    }
}