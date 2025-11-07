# Cluster Mode Documentation

## Overview

The GLIROR cluster mode enables distributed attack operations across multiple machines using a master-worker architecture. This mode allows you to coordinate attacks from multiple nodes simultaneously, significantly increasing the total request volume and distribution.

## Architecture

The cluster mode follows a master-worker pattern:

- **Master Node**: Coordinates the overall attack, distributes tasks to workers, and aggregates results
- **Worker Nodes**: Execute attack tasks assigned by the master and report back results
- **Coordinator**: HTTP server that manages communication between master and workers

## Getting Started

### Prerequisites

- All machines must be able to communicate over the network
- Master node must be accessible by all worker nodes
- Same version of GLIROR installed on all nodes

### Basic Setup

### Starting the Master Node

```bash
cargo run -- --cluster-mode --role master --url <target_url> --time <duration> --total-workers <number_of_workers> --port <port_number>
```

### Starting Worker Nodes

```bash
cargo run -- --cluster-mode --role worker --coordinator-addr "http://<master_ip>:<port>" --worker-id "worker1"
```

## Command Line Options

### Cluster Mode Options

- `--cluster-mode`: Enable cluster mode
- `--role <role>`: Specify node role ('master' or 'worker')
- `--total-workers <number>`: Number of expected worker nodes (master only)
- `--coordinator-addr <address>`: Coordinator address in format `http://host:port` (worker only)
- `--worker-id <id>`: Unique identifier for the worker (worker only)
- `-p, --port <port>`: Port for the master coordinator (default: 8080)

### Standard Attack Options (Master Only)

All standard GLIROR attack options are supported on the master node:

- `-u, --url <URL>`: Target URL to attack
- `-t, --time <seconds>`: Duration of the attack in seconds (0 for unlimited)
- `-m, --method <method>`: HTTP method to use (GET, POST, PUT, DELETE)
- `-H, --header <header>`: Custom headers (format: "Header-Name: value")
- `-D, --data <data>`: Request payload/data to send
- `-x, --proxy <proxy>`: Proxy to use for requests (format: "http://proxy:port")
- `-c, --concurrent <number>`: Number of concurrent requests
- `-w, --delay <milliseconds>`: Delay between requests in milliseconds
- `-o, --output <file>`: Output results to a file (JSON format)
- `-r, --ramp-up <seconds>`: Ramp-up time to gradually increase requests
- `-s, --schedule <time>`: Scheduled start time

## Configuration Examples

### Simple Local Cluster

**Master (Terminal 1):**
```bash
cargo run -- --cluster-mode --role master --url http://example.com --time 60 --total-workers 2 --port 8080
```

**Worker 1 (Terminal 2):**
```bash
cargo run -- --cluster-mode --role worker --coordinator-addr "http://localhost:8080" --worker-id "worker1"
```

**Worker 2 (Terminal 3):**
```bash
cargo run -- --cluster-mode --role worker --coordinator-addr "http://localhost:8080" --worker-id "worker2"
```

### Multi-Machine Cluster

**Master (on 192.168.1.100):**
```bash
cargo run -- --cluster-mode --role master --url http://target.com --time 300 --total-workers 5 --port 9000
```

**Worker 1 (on any machine):**
```bash
cargo run -- --cluster-mode --role worker --coordinator-addr "http://192.168.1.100:9000" --worker-id "worker1"
```

**Worker 2 (on any machine):**
```bash
cargo run -- --cluster-mode --role worker --coordinator-addr "http://192.168.1.100:9000" --worker-id "worker2"
```

## Master Node Features

### Task Distribution

- The master distributes attack commands to all registered workers
- Concurrent requests are automatically divided among available workers
- Each worker receives a portion of the total concurrent requests to ensure even distribution

### Real-time Monitoring

- Master displays aggregated statistics from all workers
- Shows individual worker status (INIT, READY, WORKING, COMPLETED)
- Updates in real-time with current attack metrics

### Dynamic Configuration

- If URL or duration are not specified in the command, the master will prompt for them
- Supports all standard GLIROR attack parameters
- Automatic parameter distribution to workers

## Worker Node Features

### Resilient Connection

- Automatically reconnects to master if connection is lost
- Continues operation after master restart
- Maintains persistent connection until explicitly stopped

### Task Execution

- Waits for tasks from the master after registration
- Executes assigned attack with specified parameters
- Reports real-time statistics back to master

### Status Reporting

- Sends periodic progress updates to master
- Updates status (Working, Completed, etc.)
- Individual status display with worker ID prefix

## Network Communication

### Coordinator Server

- Runs on master node to coordinate workers
- Uses HTTP/JSON API for communication
- Default port is 8080, customizable with `--port`

### API Endpoints

The coordinator exposes the following endpoints:

- `GET /`: Health check
- `POST /workers`: Register worker
- `POST /workers/:id/status`: Update worker status
- `POST /workers/:id/progress`: Report worker progress
- `GET /status`: Get cluster status
- `POST /attack`: Start attack (internal use)
- `GET /workers/:id/task`: Get task assignment

## Performance Considerations

### Optimal Worker Setup

- More workers increase attack capacity and distribution
- Consider network bandwidth limitations per worker node
- Balance number of workers with available system resources

### Concurrent Request Distribution

- Total concurrent requests are divided among workers
- Each worker gets a proportional share of the total load
- Example: 1000 total concurrent requests with 4 workers → ~250 per worker

## Error Handling

### Master Errors

- Port already in use: Clear error message with exit
- Worker connection issues: Continue operation with available workers
- Network errors: Graceful handling with status updates

### Worker Errors

- Master connectivity loss: Automatic reconnection attempts
- Task parsing errors: Continue polling for new tasks
- Network failures: Persistent reconnection with exponential backoff

## Monitoring and Status

### Master Status Display

The master shows aggregated statistics in real-time:
- Total sent requests across all workers
- Total successful requests
- Total errors across all workers
- Combined RPS (Requests Per Second)
- Average response time across all workers

### Worker Status Display

Each worker shows individual statistics:
- Individual sent requests
- Individual success/error counts
- Individual RPS
- Individual average response time
- Status prefix with worker ID

## Security Considerations

### Network Security

- By default, no authentication is required
- Coordinator server binds to localhost (127.0.0.1) by default
- When accessing from external machines, ensure network security

### Access Control

- Workers connect using worker IDs
- No authentication mechanisms are built-in
- Use network-level security (firewalls, VPNs) for production deployments

## Troubleshooting

### Common Issues

**Workers cannot connect to master:**
- Check that the coordinator address is correct
- Verify the master is running and listening on the specified port
- Ensure network connectivity between nodes

**Master shows no workers connected:**
- Verify worker command includes correct coordinator address
- Check that worker IDs are unique
- Ensure same GLIROR version on all nodes

**Performance issues:**
- Check network bandwidth between nodes
- Monitor system resources on all nodes
- Consider reducing concurrent requests per worker

### Debugging

If cluster mode fails to work as expected:

1. Start master node first
2. Verify master server is running and listening
3. Start worker nodes one by one
4. Check that all nodes can reach the coordinator address
5. Check the port is accessible from worker nodes

## Best Practices

- Use dedicated machines for worker nodes when possible
- Ensure stable network connectivity between nodes
- Monitor system resources to avoid overloading worker machines
- Start with fewer workers and scale up gradually
- Test with lower attack parameters before full-scale operations
- Consider the target's capacity and legal requirements

## Limitations

- No built-in authentication between master and workers
- Coordinator server not designed for direct internet exposure
- Network failures might temporarily interrupt coordination
- Performance depends on network connectivity between nodes