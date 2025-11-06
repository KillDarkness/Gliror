# Features

This document details all the features available in GLIROR and how to use them effectively.

## HTTP Methods

GLIROR supports multiple HTTP methods:

- **GET** (default): Retrieve data from the server
- **POST**: Send data to the server
- **PUT**: Update data on the server
- **DELETE**: Remove data from the server
- **PATCH**: Partially update data on the server
- **HEAD**: Retrieve headers without body

### Example
```bash
gliror -u https://example.com/api/users -m POST -t 60
```

## Custom Headers

Add custom headers to your requests using the `-H` or `--header` option:

```bash
gliror -u https://example.com -H "User-Agent: CustomBot/1.0" -H "Accept: application/json" -t 30
```

### Common Headers
- `User-Agent`: Identify the client
- `Authorization`: Authenticate requests
- `Content-Type`: Specify payload format
- `Accept`: Specify acceptable response format
- `X-Forwarded-For`: Spoof IP address (if server respects it)

## Request Payloads

Send data in requests using the `-D` or `--data` option:

```bash
# JSON data for POST/PUT requests
gliror -u https://api.example.com/users -m POST -D '{"name":"John","email":"john@example.com"}' -t 60

# Form data
gliror -u https://example.com/form -m POST -D 'field1=value1&field2=value2' -t 30
```

## Proxy Support

Route requests through proxy servers using the `-x` or `--proxy` option:

```bash
gliror -u https://example.com -x http://proxy-server:8080 -t 60
```

Supported proxy types:
- HTTP proxies
- HTTPS proxies
- SOCKS proxies (if supported by reqwest)

## Concurrency Control

Control the number of concurrent requests with the `-c` or `--concurrent` option:

```bash
# Low concurrency (5 requests at a time)
gliror -u https://example.com -c 5 -t 60

# High concurrency (200 requests at a time)
gliror -u https://example.com -c 200 -t 60
```

### Default Behavior
- Timed attacks: 20 concurrent requests
- Unlimited attacks: 100 concurrent requests

## Request Delay

Add delays between requests using the `-w` or `--delay` option:

```bash
# 500ms delay between each request
gliror -u https://example.com -w 500 -t 60
```

## Real-time Statistics

GLIROR provides live statistics during attacks:

- **Total Requests Sent**: Count of all requests attempted
- **Successful Requests**: Requests that returned valid responses
- **Failed Requests**: Requests that failed (timeouts, errors, etc.)
- **Requests Per Second (RPS)**: Real-time rate of requests
- **Average Response Time**: Average time taken for responses

## Error Detection

GLIROR automatically monitors for:

- **High Error Rates**: When error rate exceeds 10%
- **Slow Requests**: When individual requests take more than 2 seconds
- **Slow Average Response**: When average response time exceeds 1500ms

## Color-Coded Interface

- **Green**: Successful operations and status
- **Yellow**: Warnings and alerts
- **Blue**: Informational messages
- **Cyan**: ASCII art and major headings

## Flexible Timing

- **Limited Duration**: Set specific time in seconds
- **Unlimited**: Set time to 0 for continuous attack until interrupted
- **Default**: 30 seconds if not specified

## Command Line Interface

- **Comprehensive Options**: All features accessible via CLI
- **Help System**: Built-in help with `--help`
- **Version Information**: Version info with `--version` (no ASCII art)
- **Interactive Mode**: Prompts when required parameters are missing
- **Results Output**: Save structured results to JSON files with `--output`

## Results Output

GLIROR can save test results to structured JSON files using the `--output` option:

```bash
# Save results to a JSON file
gliror -u https://example.com -c 50 -t 120 -o results.json

# The output file contains structured data including:
# - Total requests sent
# - Successful and failed requests
# - Success rate percentage
# - Average requests per second
# - Average response time
# - Duration of the test
# - Target URL and HTTP method
```