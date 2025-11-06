# Advanced Usage

This guide covers advanced techniques and best practices for using GLIROR effectively.

## Performance Tuning

### Optimizing Concurrency
Finding the right balance between concurrency and performance:

```bash
# Conservative approach (for sensitive targets)
gliror -u https://example.com -c 10 -w 100 -t 120

# Moderate approach (for performance testing)
gliror -u https://example.com -c 50 -w 50 -t 120

# Aggressive approach (for stress testing)
gliror -u https://example.com -c 200 -t 120
```

### Delay Strategy
Using delays for different scenarios:
- **Low delay (0-50ms)**: Maximum pressure testing
- **Medium delay (50-200ms)**: Sustained load testing
- **High delay (200ms+)**: Realistic user simulation

## Complex Attack Patterns

### API Testing
Testing REST APIs with various endpoints and methods:

```bash
# Test API endpoints with authentication
gliror -u https://api.example.com/v1/users -m GET -H "Authorization: Bearer token" -c 20 -t 60

# Test API creation endpoint
gliror -u https://api.example.com/v1/users -m POST -H "Content-Type: application/json" -D '{"name":"test","email":"test@example.com"}' -c 10 -t 60

# Test API update endpoint
gliror -u https://api.example.com/v1/users/123 -m PUT -H "Content-Type: application/json" -D '{"name":"updated"}' -c 5 -t 60
```

### Form Submission Testing
Simulating form submissions:

```bash
# Test login form
gliror -u https://example.com/login -m POST -D "username=test&password=test" -H "Content-Type: application/x-www-form-urlencoded" -c 10 -t 60

# Test file upload endpoint (simulate with appropriate headers)
gliror -u https://example.com/upload -m POST -H "Content-Type: multipart/form-data" -c 5 -t 60
```

## Proxy Chains and Rotation

Using multiple proxies for advanced scenarios:

```bash
# You can create a script to rotate proxies
#!/bin/bash
PROXIES=("http://proxy1:8080" "http://proxy2:8080" "http://proxy3:8080")

for proxy in "${PROXIES[@]}"; do
  echo "Testing with proxy: $proxy"
  gliror -u https://example.com -x $proxy -c 10 -t 30
  sleep 5
done
```

## Monitoring and Analysis

### System Resource Monitoring
Monitor your system during tests:

```bash
# Monitor with htop or top in another terminal
htop
# Or monitor network usage
iftop
# Or monitor specific process
pidstat -u -p $(pgrep gliror) 1
```

### Log Analysis
For longer tests, redirect output to log files:

```bash
# Log output to file
gliror -u https://example.com -c 50 -t 300 2>&1 | tee gliror-test.log

# Then analyze the log
grep "ERROR" gliror-test.log
grep "WARNING" gliror-test.log
```

## Integration with Other Tools

### With Monitoring Tools
```bash
# Run with monitoring in background
gliror -u https://example.com -c 100 -t 300 &
MONITOR_PID=$!

# Monitor network traffic with tcpdump
tcpdump -i any -w gliror-traffic.pcap &
TCPDUMP_PID=$!

# Wait for completion
wait $MONITOR_PID
kill $TCPDUMP_PID
```

### As Part of Larger Tests
```bash
#!/bin/bash
# Comprehensive test script
TARGET="https://example.com"

echo "Starting baseline performance test..."
gliror -u $TARGET -c 10 -t 60 --output baseline.json

echo "Starting stress test..."
gliror -u $TARGET -c 100 -t 300 --output stress.json

echo "Starting endurance test..."
gliror -u $TARGET -c 50 -t 600 --output endurance.json

echo "All tests completed!"
```

## Troubleshooting Common Issues

### Connection Issues
If experiencing connection errors:

```bash
# Reduce concurrency to reduce connection pressure
gliror -u https://example.com -c 5 -t 60

# Add delays to allow connections to close
gliror -u https://example.com -c 10 -w 200 -t 60
```

### Rate Limiting
If encountering rate limits:

```bash
# Add delays to stay under rate limits
gliror -u https://api.example.com -w 1000 -c 1 -t 120

# Reduce concurrency
gliror -u https://example.com -c 1 -t 60
```

## Security Considerations

### For Authorized Testing
- Always obtain explicit written permission
- Test only systems you own or operate
- Respect rate limits where possible
- Document testing procedures and results
- Share findings appropriately

### Protecting Your Identity
```bash
# Use VPN or proxy for anonymity
gliror -u https://example.com -x http://anonymous-proxy:8080 -c 10 -t 60

# Rotate user agents
gliror -u https://example.com -H "User-Agent: CustomBot-$(date +%s)" -c 5 -t 60
```

## Benchmarking Scenarios

### Comparing Performance
```bash
# Compare different configurations
time gliror -u https://example.com -c 20 -t 30
time gliror -u https://example.com -c 50 -t 30
time gliror -u https://example.com -c 100 -t 30
```

### Infrastructure Testing
```bash
# Test load balancer behavior
gliror -u https://load-balancer.example.com -c 200 -t 300

# Test CDN performance
gliror -u https://cdn.example.com/assets -c 100 -t 120
```

## Automation Scripts

### Scheduled Testing
```bash
# crontab entry for daily testing
# 0 2 * * * /path/to/gliror -u https://example.com -c 10 -t 60

# Or using a wrapper script
#!/bin/bash
DATE=$(date +%Y-%m-%d_%H-%M)
gliror -u https://example.com -c 50 -t 300 --output "test_$DATE.json"
```

Remember to always test responsibly and in compliance with applicable laws and regulations.