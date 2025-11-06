# GLIROR Overview

GLIROR (Global Load and Intensive Request Orchestration Robot) is an advanced, high-performance DoS/load testing tool written in Rust. It's designed for authorized penetration testing and system stress testing.

## Key Features

- **High Performance**: Capable of sending thousands of concurrent requests per second
- **Multiple HTTP Methods**: Supports GET, POST, PUT, DELETE, PATCH, HEAD methods
- **Custom Headers**: Add custom headers to requests
- **Request Payloads**: Send data in requests (for POST, PUT, etc.)
- **Proxy Support**: Route requests through proxy servers
- **Rate Control**: Adjustable concurrent requests and delays
- **Real-time Statistics**: Live monitoring with RPS (requests per second)
- **Error Detection**: Monitors for high error rates and slow responses
- **Configurable Duration**: Set attack duration or run unlimited
- **Colorful Interface**: Visual feedback with colored status updates

## Architecture

GLIROR uses an asynchronous architecture based on Tokio runtime to handle high concurrency. The tool is modular with separate components for:

- CLI argument parsing
- HTTP request handling
- Display and user interface
- Statistics and monitoring
- Utility functions

## Use Cases

- Load testing your own applications
- Authorized penetration testing
- Infrastructure stress testing
- Performance benchmarking
- Network resilience testing

> **Note**: GLIROR is intended for authorized testing only. Always ensure you have explicit permission before testing systems you do not own.