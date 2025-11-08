# Configuration File

GLIROR supports loading attack parameters from a YAML configuration file, making it easier to manage and reuse complex attack scenarios.

## Usage

To use a configuration file, specify its path using the `--config` command-line flag:

```bash
gliror --config /path/to/your/config.yml
```

## Format and Fields

The configuration file must be in YAML format. All fields are optional. Below is a complete example with all available fields and explanatory comments.

## Full Example Configuration

```yaml
# Example configuration file for GLIROR
# All fields are optional. Command-line arguments will override these values.

# --- Target ---
# url: "http://example.com"
host: "example.com"
target_port: 80

# --- Attack Type ---
# "http", "udp", or "slowloris"
attack_type: "http"

# --- Attack Parameters ---
# Duration in seconds (0 for unlimited)
time: 60
# Number of concurrent requests/threads
concurrent: 100
# Delay between each request in milliseconds
delay: 0
# Ramp-up time in seconds
ramp_up: 10

# --- HTTP Specifics ---
# "GET", "POST", "PUT", "DELETE", etc.
method: "GET"
# Request body/data
data: '{"key": "value"}'
# Custom headers
headers:
  Content-Type: "application/json"
  X-Custom-Header: "gliror-ddos"

# --- Network ---
# Proxy server (e.g., "http://user:pass@host:port")
# proxy: "http://127.0.0.1:8080"

# --- Scheduling ---
# Start time in UTC "YYYY-MM-DD HH:MM:SS" or delay in seconds
# schedule: "2025-12-31 23:59:59"

# --- Cluster Mode ---
# cluster_mode: true
# role: "master" # or "worker"
# port: 9000 # Port for the master to listen on
# coordinator_addr: "http://localhost:9000" # For workers
# total_workers: 2
```

## Precedence

The configuration is loaded as a baseline. **Any argument provided directly on the command line will override the corresponding value from the configuration file.**

For example, if your `config.yml` specifies `time: 120`, but you run the command:

```bash
gliror --config config.yml --time 30
```

The attack will run for **30 seconds**, as the command-line flag takes precedence.
