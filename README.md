# GLIROR

A high-performance DoS (Denial of Service) tool written in Rust with colorful status display, error detection, and performance monitoring.

## Documentation

Comprehensive documentation is available in the [docs](docs/) directory:

- [Overview](docs/overview.md) - Introduction and core concepts
- [Installation](docs/installation.md) - How to install and set up GLIROR
- [Usage](docs/usage.md) - Detailed usage instructions and examples
- [Features](docs/features.md) - In-depth feature explanations
- [Advanced Usage](docs/advanced.md) - Advanced techniques and best practices
- [Troubleshooting](docs/troubleshooting.md) - Common issues and solutions
- [Performance Tips](docs/performance.md) - Optimizing GLIROR performance
- [Legal Notice](docs/legal.md) - Important legal information
- [FAQ](docs/faq.md) - Frequently asked questions

## Features

- **ASCII Art Display**: Shows "KILLDOS" in cyan ASCII art at startup (intentionally kept as requested)
- **Concurrent Requests**: Sends thousands of requests per second using async/await
- **Configurable Duration**: Set attack duration or run unlimited
- **Real-time Status**: Displays live statistics with colors
- **Error Detection**: Monitors for high error rates and alerts user
- **Performance Monitoring**: Tracks average response times and warns of slow requests
- **HTTP Method Selection**: Support for GET, POST, PUT, DELETE, PATCH, HEAD methods
- **Custom Headers**: Add custom headers to requests
- **Request Payloads**: Send data in requests (for POST, PUT, etc.)
- **Proxy Support**: Route requests through proxies
- **Rate Control**: Adjustable concurrent requests and delays
- **Two Modes**: 
  - **Fast Mode**: 100 concurrent requests for maximum load (when time=0)
  - **Medium Mode**: 20 concurrent requests for sustained attack

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd gliror

# Build the project
cargo build --release

# Run the tool
cargo run --release -- --url <target-url> --time <duration-in-seconds>
```

## Usage

### Command Line Options

```bash
Usage: gliror [OPTIONS]

Options:
  -u, --url <URL>                Target URL to attack
  -t, --time <TIME>              Duration of the attack in seconds (0 for unlimited) [default: 30]
  -m, --method <METHOD>          HTTP method to use (GET, POST, PUT, DELETE) [default: GET]
  -H, --header <HEADER>          Custom headers (format: "Header-Name: value")
  -D, --data <DATA>              Request payload/data to send
  -x, --proxy <PROXY>            Proxy to use for requests (format: "http://proxy:port")
  -c, --concurrent <CONCURRENT>  Number of concurrent requests (default: 100 for unlimited, 20 for timed)
  -w, --delay <DELAY>            Delay between requests in milliseconds [default: 0]
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples

1. **Basic Usage**:
   ```bash
   cargo run --release -- --url https://example.com --time 60
   ```

2. **Unlimited Attack**:
   ```bash
   cargo run --release -- --url https://example.com --time 0
   ```

3. **POST Request with Data**:
   ```bash
   cargo run --release -- --url https://example.com/api --method POST --data '{"key": "value"}' --time 30
   ```

4. **With Custom Headers**:
   ```bash
   cargo run --release -- --url https://example.com --header "User-Agent: CustomBot/1.0" --header "Accept: application/json" --time 60
   ```

5. **Through a Proxy**:
   ```bash
   cargo run --release -- --url https://example.com --proxy http://127.0.0.1:8080 --time 60
   ```

6. **With Custom Concurrency and Delay**:
   ```bash
   cargo run --release -- --url https://example.com --concurrent 50 --delay 100 --time 60
   ```

7. **Interactive Mode** (without command line parameters):
   ```bash
   cargo run --release
   # Then follow the prompts to enter URL and duration
   ```

## Statistics Displayed

During the attack, GLIROR shows:

- Total requests sent
- Successful requests
- Failed requests
- Average requests per second (RPS)
- Average response time
- Current status message

## Warnings

The tool provides real-time warnings for:

- High error rates (over 10%)
- Slow requests (over 2 seconds individually)
- High average response times (over 1500ms)

## Dependencies

- `tokio` - Async runtime for concurrent requests
- `reqwest` - HTTP client for making requests
- `clap` - Command line argument parsing
- `colored` - Colored terminal output
- `indicatif` - Progress bar and status display

## Important Note

This tool is intended for educational purposes and authorized penetration testing only. Misuse of this tool against systems without explicit permission may be illegal and could cause service disruptions. Always obtain proper authorization before testing. See our [Legal Notice](docs/legal.md) for complete legal information.

## License

This project is licensed under the MIT License - see the LICENSE file for details.