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
use cluster::{ClusterCoordinator, ClusterConfig};

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
        let coordinator_addr = args_clone.coordinator_addr.unwrap_or_else(|| "http://localhost:8080".to_string());
        let total_workers = args_clone.total_workers.unwrap_or(2);
        let worker_id = args_clone.worker_id.unwrap_or_else(|| {
            use uuid::Uuid;
            Uuid::new_v4().to_string()
        });
        
        let config = ClusterConfig::new(total_workers, coordinator_addr);
        
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

async fn run_master_node(args: Args) {
    let args_clone = args.clone();
    let config = ClusterConfig::new(
        args_clone.total_workers.unwrap_or(2),
        args_clone.coordinator_addr.unwrap_or_else(|| "http://localhost:8080".to_string())
    );
    
    let coordinator = ClusterCoordinator::new(config);
    println!("{} Master node initialized, coordinating {} workers", "INFO".blue(), coordinator.config.total_workers);
    
    // For now, just delegate to regular attack with distributed load
    let mut modified_args = args.clone();
    if let Some(mut concurrent) = modified_args.concurrent {
        concurrent = (concurrent / args_clone.total_workers.unwrap_or(1) as u32).max(1);
        modified_args.concurrent = Some(concurrent);
    }
    
    handle_standalone_attack(modified_args).await;
}

async fn run_worker_node(args: Args) {
    let args_clone = args.clone();
    let worker_id = args_clone.worker_id.unwrap_or_else(|| {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    });
    
    println!("{} Worker node {} registered with coordinator", "INFO".blue(), worker_id);
    
    // For now, just delegate to regular attack with distributed load
    let mut modified_args = args;
    if let Some(mut concurrent) = modified_args.concurrent {
        concurrent = (concurrent / args_clone.total_workers.unwrap_or(1) as u32).max(1);
        modified_args.concurrent = Some(concurrent);
    }
    
    handle_standalone_attack(modified_args).await;
}