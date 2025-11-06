use colored::Colorize;
use std::io::{self, Write};

pub fn prompt_for_url() -> String {
    print!("{} ", "Enter target URL:".cyan());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

pub fn prompt_for_time() -> u64 {
    print!("{} ", "Enter duration in seconds (0 for unlimited): ".cyan());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap_or(30)
}