<div align="center">
  <img src="https://i.imgur.com/uqXfbxe.jpeg" alt="GLIROR Banner">
  <h1>GLIROR</h1>
  <p>
    <strong>An advanced, multi-vector (HTTP, UDP, Slowloris) DoS and Load Testing tool with cluster support, written in Rust.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/gliror"><img src="https://img.shields.io/crates/v/gliror.svg" alt="Crates.io"></a>
    <a href="https://github.com/KillDarkness/Gliror/blob/main/docs/index.md"><img src="https://img.shields.io/badge/docs-main-blue.svg" alt="Documentation"></a>
    <a href="https://github.com/KillDarkness/Gliror/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  </p>
</div>

GLIROR is a high-performance Denial of Service and load testing tool designed for security professionals and developers. It supports multiple attack vectors, real-time performance monitoring, and can be run in a distributed cluster for large-scale tests.

## Features

- **Multiple Attack Vectors**: Supports HTTP/S flooding, UDP flooding, and Slowloris attacks.
- **Cluster Mode**: Distribute load across multiple worker nodes, controlled by a central master.
- **YAML Configuration**: Define and manage complex attack profiles using simple YAML files.
- **Real-time Monitoring**: Colorful, real-time status display of key metrics like RPS, success/error rates, and average response time.
- **Advanced HTTP Control**: Customize HTTP method, headers, and request payloads.
- **Network Flexibility**: Route traffic through HTTP/S proxies.
- **Rate & Duration Control**: Fine-tune concurrency, delay, and attack duration.
- **Multi-format Output**: Save comprehensive test results to files in JSON, XML, YAML, CSV, or TOML formats for analysis.

## Installation

Ensure you have the Rust toolchain installed. Then, install GLIROR from Crates.io:

```bash
cargo install gliror
```

## Quick Start

### Example 1: Simple HTTP Flood

```bash
gliror --url http://example.com --time 60 --concurrent 100
```

### Example 2: UDP Flood

```bash
gliror --attack-type udp --host 1.1.1.1 --target-port 53 --time 60
```

### Example 3: Using a Configuration File

Create a file named `attack.yml`:
```yaml
url: "http://example.com"
attack_type: "http"
time: 120
concurrent: 200
headers:
  X-Custom-Header: "gliror-test"
```

Run the attack:
```bash
gliror --config attack.yml
```

### Example 4: Configuration File with Cluster Distribution Mode

Create a file named `cluster_attack.yml`:
```yaml
url: "http://example.com"
attack_type: "http"
time: 120
concurrent: 200
cluster_mode: true
role: "master"
distribution_mode: "max-power"  # Options: "even" (default), "max-power"
total_workers: 2
port: 8080
output: "results.json"  # Output file for worker results
```

Run the master node:
```bash
gliror --config cluster_attack.yml --role master
```

### Example 5: Cluster Mode with Max Power Distribution

Run the master node with max power distribution mode:
```bash
gliror --cluster-mode --role master --distribution-mode max-power --url http://example.com --time 60 --concurrent 1000
```

Run worker nodes:
```bash
gliror --cluster-mode --role worker --coordinator-addr http://MASTER_IP:8080
```

This will distribute the full concurrent load to each worker, allowing for maximum request throughput across the cluster.

## Documentation

For detailed information, please refer to the documentation in the `docs/` directory.

1.  [Overview](docs/overview.md)
2.  [Installation](docs/installation.md)
3.  [Usage](docs/usage.md)
4.  [Configuration File](docs/configuration.md)
5.  [Features](docs/features.md)
6.  [Advanced Usage](docs/advanced.md)
7.  [Cluster Mode](docs/cluster.md)
8.  [Troubleshooting](docs/troubleshooting.md)
9.  [Performance Tips](docs/performance.md)
10. [Legal Notice](docs/legal.md)
11. [FAQ](docs/faq.md)

## Legal Disclaimer

This tool is intended for educational purposes and authorized security testing only. Misuse of this tool against systems without explicit permission is illegal. The developers assume no liability and are not responsible for any misuse or damage caused by this program.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.