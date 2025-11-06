![GLIROR](https://i.imgur.com/uqXfbxe.jpeg)

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

- **ASCII Art Display**: Shows ASCII art in cyan at startup (intentionally kept as requested)
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
- **Results Output**: Save structured test results to JSON files
- **Two Modes**: 
  - **Fast Mode**: 100 concurrent requests for maximum load (when time=0)
  - **Medium Mode**: 20 concurrent requests for sustained attack

## Important Note

This tool is intended for educational purposes and authorized penetration testing only. Misuse of this tool against systems without explicit permission may be illegal and could cause service disruptions. Always obtain proper authorization before testing. See our [Legal Notice](docs/legal.md) for complete legal information.

## License

This project is licensed under the MIT License - see the LICENSE file for details.