use clap::Parser;
use std::collections::HashMap;

// Parse custom headers from command line arguments
pub fn parse_headers(header_args: &[String]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    for header_str in header_args {
        if let Some((name, value)) = header_str.split_once(':') {
            headers.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    
    headers
}

#[derive(Parser)]
#[clap(
    name = "GLIROR",
    about = "High-performance DoS tool with colorful status display",
    author = "GLIROR Team",
    version = "1.0.1"
)]
pub struct Args {
    /// Target URL to attack
    #[clap(short, long, value_parser)]
    pub url: Option<String>,
    
    /// Duration of the attack in seconds (0 for unlimited)
    #[clap(short, long, value_parser, default_value_t = 30)]
    pub time: u64,
    
    /// HTTP method to use (GET, POST, PUT, DELETE)
    #[clap(short, long, value_parser, default_value = "GET")]
    pub method: String,
    
    /// Custom headers (format: "Header-Name: value")
    #[clap(short = 'H', long, value_parser)]
    pub header: Vec<String>,
    
    /// Request payload/data to send
    #[clap(short = 'D', long, value_parser)]
    pub data: Option<String>,
    
    /// Proxy to use for requests (format: "http://proxy:port")
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
}