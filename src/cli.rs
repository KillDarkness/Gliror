use clap::Parser;
use std::collections::HashMap;

pub fn parse_headers(header_args: &[String]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    for header_str in header_args {
        if let Some((name, value)) = header_str.split_once(':') {
            headers.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    
    headers
}

#[derive(Parser, Clone)]
#[clap(
    name = "GLIROR",
    about = "High-performance DoS tool with colorful status display",
    author = "GLIROR Team",
    version = "1.0.4"
)]
pub struct Args {
    /// Target URL to attack (for HTTP attacks) or host:port for UDP attacks
    #[clap(short, long, value_parser)]
    pub url: Option<String>,
    
    /// Target host for UDP attacks (alternative to URL)
    #[clap(short = 'S', long, value_parser)]
    pub host: Option<String>,
    
    /// Target port for UDP attacks
    #[clap(short = 'T', long, value_parser)]
    pub target_port: Option<u16>,
    
    /// Attack type (http, udp, or slowloris)
    #[clap(short = 'A', long, value_parser, default_value = "http")]
    pub attack_type: String,
    
    /// Duration of the attack in seconds (0 for unlimited)
    #[clap(short, long, value_parser, default_value_t = 30)]
    pub time: u64,
    
    /// HTTP method to use (GET, POST, PUT, DELETE) - ignored for UDP
    #[clap(short, long, value_parser, default_value = "GET")]
    pub method: String,
    
    /// Custom headers (format: "Header-Name: value") - ignored for UDP
    #[clap(short = 'H', long, value_parser)]
    pub header: Vec<String>,
    
    /// Request payload/data to send
    #[clap(short = 'D', long, value_parser)]
    pub data: Option<String>,
    
    /// Proxy to use for requests (format: "http://proxy:port") - ignored for UDP
    #[clap(short = 'x', long, value_parser)]
    pub proxy: Option<String>,
    
    /// Number of concurrent requests (default: 100 for unlimited, 20 for timed)
    #[clap(short = 'c', long, value_parser)]
    pub concurrent: Option<u32>,
    
    /// Delay between requests in milliseconds
    #[clap(short = 'w', long, value_parser, default_value_t = 0)]
    pub delay: u64,
    
    /// Output results to a file (JSON format)
    #[clap(short = 'o', long, value_parser)]
    pub output: Option<String>,
    
    /// Ramp-up time in seconds (gradually increase to target concurrent requests)
    #[clap(short = 'r', long, value_parser)]
    pub ramp_up: Option<u64>,
    
    /// Scheduled start time in UTC (format: "YYYY-MM-DD HH:MM:SS" in UTC timezone or delay in seconds)
    #[clap(short = 's', long, value_parser)]
    pub schedule: Option<String>,
    
    /// Enable cluster mode
    #[clap(long)]
    pub cluster_mode: bool,
    
    /// Worker ID for cluster mode
    #[clap(long, value_parser)]
    pub worker_id: Option<String>,
    
    /// Coordinator address for cluster mode
    #[clap(long, value_parser)]
    pub coordinator_addr: Option<String>,
    
    /// Number of cluster workers
    #[clap(long, value_parser)]
    pub total_workers: Option<usize>,
    
    /// Port for the master coordinator (default: 8080)
    #[clap(short = 'p', long, value_parser)]
    pub port: Option<u16>,

    /// Cluster worker role ('master' or 'worker')
    #[clap(long, value_parser)]
    pub role: Option<String>,
}