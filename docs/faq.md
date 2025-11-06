# Frequently Asked Questions (FAQ)

This document answers common questions about GLIROR usage, functionality, and troubleshooting.

## General Questions

### Q: What is GLIROR?
A: GLIROR (Global Load and Intensive Request Orchestration Robot) is an advanced, high-performance DoS/load testing tool written in Rust. It's designed for authorized penetration testing and system stress testing.

### Q: Is GLIROR a DDoS tool?
A: GLIROR is designed for load testing and authorized penetration testing. It is a single-instance tool that runs on one system. It should only be used on systems you own or have explicit permission to test. Any use against systems without authorization is illegal and unethical.

### Q: What programming language is GLIROR written in?
A: GLIROR is written in Rust, which provides memory safety, performance, and modern systems programming capabilities.

### Q: Is GLIROR open source?
A: Yes, GLIROR is open source and available under the MIT license.

## Installation and Setup

### Q: What are the system requirements for GLIROR?
A: 
- Rust 1.70 or higher
- Operating system: Linux, macOS, or Windows with WSL/cross-compilation support
- At least 2GB RAM recommended
- Sufficient network bandwidth for your testing needs

### Q: How do I install GLIROR?
A: See the [Installation](installation.md) guide for detailed instructions. The most common method is to build from source using Cargo.

### Q: Why do I get OpenSSL errors during compilation?
A: Install OpenSSL development libraries:
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# CentOS/RHEL/Fedora
sudo dnf install openssl-devel pkgconfig
```

### Q: Can I install GLIROR system-wide?
A: Yes, after building, you can install system-wide with:
```bash
cargo install --path .
```

## Usage Questions

### Q: How do I run a basic test?
A: 
```bash
gliror -u https://example.com -t 30
```
This runs a 30-second test against the specified URL with default settings.

### Q: How do I send POST requests with data?
A: 
```bash
gliror -u https://example.com/api -m POST -D '{"key":"value"}' -t 60
```
Use the `-D` flag to send data with your requests.

### Q: Can I add custom headers?
A: Yes, use the `-H` flag:
```bash
gliror -u https://example.com -H "Authorization: Bearer token" -H "User-Agent: Custom" -t 30
```

### Q: What's the difference between timed and unlimited tests?
A: 
- Timed tests (`-t 30`) run for the specified duration
- Unlimited tests (`-t 0`) run continuously until stopped with Ctrl+C
- Default concurrent requests: 20 for timed, 100 for unlimited

### Q: How do I use a proxy?
A: Use the `-x` flag:
```bash
gliror -u https://example.com -x http://proxy:8080 -t 60
```

### Q: What does RPS mean in the output?
A: RPS stands for "Requests Per Second" - a measure of how many requests GLIROR is sending to the target per second.

## Performance and Optimization

### Q: How can I increase performance/concurrency?
A: You can increase the concurrent request count with:
```bash
gliror -u https://example.com -c 200 -t 60
```
Be sure to monitor your system resources when increasing concurrency.

### Q: Why is my RPS lower than expected?
A: Possible reasons include:
- Target system limiting requests
- Network bandwidth limitations
- System resource constraints
- Network latency to the target

### Q: How do I reduce the request rate?
A: Add delays between requests:
```bash
gliror -u https://example.com -w 100 -t 60  # 100ms delay
```

### Q: What's the maximum concurrency supported?
A: This depends on your system resources. Start with lower values and increase gradually while monitoring system performance.

## Troubleshooting

### Q: I'm getting "Too many open files" error. How do I fix this?
A: Increase your file descriptor limits:
```bash
ulimit -n 4096
```

### Q: Why are my requests timing out?
A: This could be due to:
- Network connectivity issues
- Target server being overloaded
- Firewall blocking requests
- Try reducing concurrency or adding delays

### Q: How do I know if my test was successful?
A: Look for:
- High success rate (>95%)
- Stable RPS values
- Acceptable response times
- No system errors on your end

### Q: GLIROR seems to hang. What should I do?
A: 
- Check if the target is still responding
- Verify your network connectivity
- Try reducing concurrency
- Use Ctrl+C to stop the test if needed

## Safety and Legal

### Q: Is it legal to use GLIROR?
A: GLIROR is legal when used for authorized testing on systems you own or have explicit permission to test. Using it against systems without authorization is illegal and unethical.

### Q: How can I ensure I'm using GLIROR legally?
A: 
- Only test systems you own or operate
- Obtain written authorization before testing others' systems
- Comply with all applicable laws in your jurisdiction
- Follow responsible disclosure practices

### Q: Can GLIROR be detected by security tools?
A: GLIROR sends standard HTTP requests, so it may be detectable by intrusion detection systems. Use only on systems you're authorized to test.

## Advanced Questions

### Q: Can I script GLIROR for automated testing?
A: Yes! GLIROR is designed to be used in scripts. You can capture output and parse results as needed.

### Q: Does GLIROR support HTTPS?
A: Yes, GLIROR supports both HTTP and HTTPS through the reqwest library.

### Q: How does GLIROR handle connection pooling?
A: GLIROR uses the reqwest library which handles connection pooling automatically for better performance.

### Q: Can I get structured output (JSON, etc.)?
A: Currently GLIROR provides text-based output. You can redirect and parse the output as needed in your scripts.

## Technical Details

### Q: How does GLIROR achieve high concurrency?
A: GLIROR uses Rust's async/await with the Tokio runtime to handle many concurrent connections efficiently.

### Q: What HTTP methods are supported?
A: GET, POST, PUT, DELETE, PATCH, and HEAD methods are supported.

### Q: Does GLIROR support HTTP/2?
A: GLIROR uses the reqwest library which supports HTTP/1.1 and HTTP/2 automatically based on server capabilities.

### Q: How can I contribute to GLIROR?
A: Contributions are welcome! Check the repository for contribution guidelines.

## Version and Updates

### Q: How do I check my GLIROR version?
A: 
```bash
gliror --version
# or
gliror -V
```

### Q: How do I update GLIROR?
A: 
```bash
# Update from source
git pull
cargo build --release
```

## Support

### Q: Where can I get help with GLIROR?
A: 
- Check the documentation
- Review the GitHub repository issues
- Ensure you're using the tool responsibly and within legal bounds

If your question isn't answered here, please refer to the complete documentation linked in the main documentation index.