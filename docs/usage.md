# Usage

This guide covers how to use GLIROR effectively with various examples and scenarios.

## Basic Syntax

```bash
gliror [OPTIONS]
```

## Available Options

### Target Configuration
- `-u, --url <URL>`: Target URL to attack (required)
- `-t, --time <TIME>`: Duration of the attack in seconds (0 for unlimited) [default: 30]

### HTTP Configuration
- `-m, --method <METHOD>`: HTTP method to use (GET, POST, PUT, DELETE, PATCH, HEAD) [default: GET]
- `-H, --header <HEADER>`: Custom headers (format: "Header-Name: value")
- `-D, --data <DATA>`: Request payload/data to send

### Connection Configuration
- `-x, --proxy <PROXY>`: Proxy to use for requests (format: "http://proxy:port")
- `-c, --concurrent <CONCURRENT>`: Number of concurrent requests [default: 20 for timed, 100 for unlimited]
- `-w, --delay <DELAY>`: Delay between requests in milliseconds [default: 0]

### Information
- `-h, --help`: Print help information
- `-V, --version`: Print version information
- `-o, --output <OUTPUT>`: Output results to a file (JSON format)

### Configuration File
- `--config <PATH>`: Path to a YAML configuration file.

You can define all attack parameters in a YAML file and load it using the `--config` flag. This is useful for managing complex or repeated attack scenarios. Command-line arguments will always override values set in the configuration file.

See the [Configuration File](configuration.md) documentation for a detailed guide and a full example.

## Usage Examples

### Basic Usage
```bash
# Basic attack for 30 seconds
gliror -u https://example.com -t 30

# Unlimited attack (Ctrl+C to stop)
gliror -u https://example.com -t 0
```

### Advanced Usage
```bash
# POST request with JSON data
gliror -u https://api.example.com/users -m POST -D '{"name":"test","email":"test@example.com"}' -t 60

# With custom headers
gliror -u https://example.com -H "User-Agent: CustomBot/1.0" -H "Accept: application/json" -t 30

# Through a proxy
gliror -u https://example.com -x http://proxy-server:8080 -t 60

# Custom concurrency and delay
gliror -u https://example.com -c 50 -w 100 -t 60

# Multiple headers and complex method
gliror -u https://api.example.com/graphql -m POST -H "Content-Type: application/json" -H "Authorization: Bearer token" -D '{"query":"{ users { id name } }"}' -t 120
```

## Interactive Mode

If you don't specify the URL and time options, GLIROR will prompt you:

```bash
gliror
# Prompts for URL and time
```

## Understanding the Output

### Status Display
- `Sent`: Total requests sent
- `Success`: Successful requests
- `Errors`: Failed requests
- `RPS`: Requests per second
- `Avg`: Average response time in milliseconds

### Warnings
- High error rate detected (over 10%)
- Slow request detected (over 2 seconds)
- Average response time is high (over 1500ms)

## Best Practices

1. **Start Small**: Begin with lower concurrency to understand the target's response
2. **Monitor**: Always monitor both target and your own system resources
3. **Authorize**: Only test systems you own or have explicit permission to test
4. **Time Appropriately**: Use limited duration for initial tests
5. **Respect Limits**: Be aware of rate limits and implement appropriate delays if needed