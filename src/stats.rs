use std::collections::HashMap;

/// Parse custom headers from command line arguments
pub fn parse_headers(header_args: &[String]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    for header_str in header_args {
        if let Some((name, value)) = header_str.split_once(':') {
            headers.insert(name.trim().to_string(), value.trim().to_string());
        }
    }
    
    headers
}