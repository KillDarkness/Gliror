mod cli;
mod http;
mod display;
mod stats;
mod utils;

use clap::Parser;
use cli::Args;
use colored::Colorize;
use display::print_cyan_ascii;
use http::perform_attack;
use cli::parse_headers;

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();
    
    // Show version without ASCII art if version flag is present
    if std::env::args().any(|arg| arg == "--version" || arg == "-V") {
        println!("GLIROR {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    
    // Display ASCII art in cyan for all other commands
    print_cyan_ascii();
    
    // Get URL from command line or prompt user
    let target_url = if let Some(url) = args.url {
        url
    } else {
        utils::prompt_for_url()
    };
    
    // Get time from command line or prompt user
    let duration = if args.time != 0 {
        args.time
    } else {
        utils::prompt_for_time()
    };
    
    // Parse headers
    let headers = parse_headers(&args.header);
    
    println!("\n{} Starting attack on: {}", "INFO".blue(), target_url);
    println!("{} Method: {}", "INFO".blue(), args.method);
    println!("{} Duration: {} seconds", "INFO".blue(), if duration == 0 { "Unlimited".to_string() } else { duration.to_string() });
    println!("{} Concurrent requests: {}", "INFO".blue(), 
             args.concurrent.unwrap_or(if duration == 0 { 100 } else { 20 }));
    if let Some(ref proxy) = args.proxy {
        println!("{} Proxy: {}", "INFO".blue(), proxy);
    }
    
    // Start the attack
    perform_attack(target_url, duration, args.method, headers, args.data, args.proxy, args.concurrent, args.delay, args.output).await;
}