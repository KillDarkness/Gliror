# Performance Tips

This guide provides optimization strategies to get the best performance from GLIROR while maintaining system stability.

## System Optimization

### Hardware Requirements

**CPU**: Multi-core processor recommended for handling concurrent requests
- More cores = better concurrent performance
- Modern CPUs with higher clock speeds improve response times

**RAM**: Minimum 2GB recommended
- High concurrency requires more memory
- Each connection consumes memory resources
- Monitor memory usage during tests

**Network**: High-speed internet connection
- Bandwidth affects overall throughput
- Low latency connections improve response times
- Consider location relative to target

### Operating System Tuning

#### Linux System Tuning

```bash
# Increase file descriptor limits
echo "* soft nofile 1048576" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 1048576" | sudo tee -a /etc/security/limits.conf

# Increase network buffer sizes
echo 'net.core.rmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem = 4096 87380 16777216' | sudo tee -a /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem = 4096 65536 16777216' | sudo tee -a /etc/sysctl.conf

# Apply changes
sudo sysctl -p
```

#### Check Current Limits

```bash
# File descriptors
ulimit -n

# Network buffers
cat /proc/sys/net/core/rmem_max
cat /proc/sys/net/core/wmem_max

# Connection tracking
cat /proc/sys/net/netfilter/nf_conntrack_max
```

## GLIROR Configuration Optimization

### Finding Optimal Concurrency

Test different concurrency levels to find the sweet spot:

```bash
# Test script to find optimal concurrency
TARGET="https://example.com"
DURATION=60

for CONCURRENCY in 10 25 50 100 200 500; do
    echo "Testing with $CONCURRENCY concurrent requests"
    gliror -u $TARGET -c $CONCURRENCY -t $DURATION 2>&1 | grep -E "(RPS|Success rate|Avg response)"
    sleep 10  # Pause between tests
done
```

### Balancing Concurrency vs Delays

#### High Performance (Aggressive)
```bash
gliror -u https://target.com -c 200 -w 0 -t 60
```

#### Balanced Performance (Recommended for most tests)
```bash
gliror -u https://target.com -c 50 -w 10 -t 60
```

#### Low Impact (For sensitive systems)
```bash
gliror -u https://target.com -c 5 -w 100 -t 60
```

## Network Optimization

### Connection Reuse

While GLIROR uses reqwest which handles connection pooling automatically, you can optimize by:

1. **Keeping tests to the same target**: Allows connection reuse
2. **Avoiding DNS lookups**: Use IP addresses if DNS is a bottleneck
3. **Using HTTP/2 if available**: More efficient for concurrent requests

### Bandwidth Management

Monitor your network during tests:
```bash
# Monitor network usage during test
iftop -i eth0  # Replace eth0 with your network interface
```

## Memory Management

### Monitoring Memory Usage

```bash
# Monitor memory usage in another terminal
watch -n 1 'ps aux | grep gliror | grep -v grep'

# Or use htop
htop
```

### Memory Optimization Settings

1. **Reduce concurrency** if memory usage is too high
2. **Add delays** to allow connections to close properly
3. **Limit test duration** for high concurrency tests

## Target-Specific Optimization

### API Endpoints

API endpoints often have specific performance characteristics:

```bash
# For API testing, consider rate limiting
gliror -u https://api.example.com/endpoint -c 20 -w 100 -t 120

# For lightweight endpoints, higher concurrency
gliror -u https://example.com/static.js -c 100 -t 60
```

### Static Content vs Dynamic Content

- **Static content**: Higher concurrency with less impact
- **Dynamic content**: Lower concurrency with database interactions

## Performance Monitoring Tools

### System Monitoring

```bash
# Monitor during tests
htop    # CPU and memory
iftop   # Network usage
iostat  # I/O statistics
pidstat # Process statistics

# Specific GLIROR monitoring
pidstat -u -r -n -p $(pgrep gliror) 1
```

### Network Analysis

```bash
# Monitor network connections
ss -s  # Connection summary
netstat -an | grep ESTABLISHED | wc -l  # Count established connections

# Packet analysis (if needed)
sudo tcpdump -i any -c 1000 host target.com
```

## Common Performance Patterns

### Gradual Load Increase

```bash
#!/bin/bash
TARGET=$1
CONCURRENCIES=(10 25 50 100 200)

for C in "${CONCURRENCIES[@]}"; do
    echo "Testing with concurrency: $C"
    RPS=$(gliror -u $TARGET -c $C -t 30 2>&1 | grep -oE '[0-9]+\.[0-9]+ RPS' | tail -1)
    echo "Result: $RPS"
    sleep 15  # Recovery time
done
```

### Endurance Testing

For long-term performance testing:

```bash
# Lower concurrency for extended testing
gliror -u https://example.com -c 30 -t 1800  # 30 minutes

# Monitor for degradation over time
gliror -u https://example.com -c 20 -t 3600  # 1 hour
```

## Performance Metrics to Watch

### Key Indicators

1. **Requests Per Second (RPS)**: Primary performance metric
2. **Success Rate**: Percentage of successful requests
3. **Average Response Time**: Indicator of system responsiveness
4. **Error Rate**: Shows system stress levels

### Good Performance Ranges

- **High RPS**: >100 RPS (varies by target)
- **Success Rate**: >95% (varies by target)
- **Response Time**: <500ms for quick responses
- **Error Rate**: <5% (varies by target)

## Optimization Strategies

### For High-Performance Testing

```bash
# System tuning + high concurrency
gliror -u https://target.com -c 150 -w 0 -t 120
```

### For Accurate Performance Measurement

```bash
# Balanced approach for accurate metrics
gliror -u https://target.com -c 50 -w 20 -t 180
```

### For Resource-Constrained Systems

```bash
# Conservative approach
gliror -u https://target.com -c 10 -w 100 -t 120
```

## Troubleshooting Performance Issues

### Low RPS

1. Check network connectivity
2. Adjust concurrency settings
3. Verify target server is not overloaded
4. Monitor system resources

### High Error Rates

1. Reduce concurrency
2. Add delays between requests
3. Check target server logs
4. Verify request format

### Memory Issues

1. Decrease concurrency
2. Run shorter tests
3. Close other applications
4. Increase system memory limits

## Best Practices

1. **Start conservative**: Begin with low concurrency and increase gradually
2. **Monitor resources**: Always monitor system resources during tests
3. **Use appropriate duration**: Balance between meaningful data and system load
4. **Test in safe environments**: Use staging systems when possible
5. **Document configurations**: Keep track of what works best for different targets

By following these performance tips, you should be able to achieve optimal performance from GLIROR while maintaining system stability and getting meaningful results.