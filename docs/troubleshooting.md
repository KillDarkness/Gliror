# Troubleshooting

This guide helps diagnose and resolve common issues with GLIROR.

## Common Issues

### Compilation Errors

**Issue**: `error: failed to run custom build command for 'openssl-sys'`
**Solution**: Install OpenSSL development libraries:
```bash
# Ubuntu/Debian
sudo apt install libssl-dev pkg-config

# CentOS/RHEL/Fedora
sudo dnf install openssl-devel pkgconfig

# macOS
brew install openssl
```

**Issue**: `error[E0433]: failed to resolve: use of undeclared type`
**Solution**: Ensure all imports are properly added in each module

### Runtime Errors

**Issue**: "Connection refused" or "Could not resolve host"
**Solution**: 
- Verify target URL is correct
- Check internet connectivity
- Ensure target server is accessible

**Issue**: "Too many open files" error
**Solution**: Increase system file descriptor limits:
```bash
# Temporary increase
ulimit -n 4096

# Permanent increase in /etc/security/limits.conf
echo "* soft nofile 4096" | sudo tee -a /etc/security/limits.conf
echo "* hard nofile 4096" | sudo tee -a /etc/security/limits.conf
```

**Issue**: High memory usage
**Solution**: Reduce concurrency:
```bash
gliror -u https://example.com -c 20 -t 60  # Lower concurrent requests
```

## Connection Problems

### Timeout Issues

**Issue**: Requests timing out frequently
**Solutions**:
1. Increase timeout value in the code
2. Reduce concurrency to reduce server load
3. Add delays between requests
```bash
gliror -u https://example.com -c 10 -w 100 -t 60  # Reduce pressure
```

### SSL/TLS Issues

**Issue**: "SSL error" or "TLS handshake failed"
**Solutions**:
1. Update system certificates
2. Try different TLS version if supported
3. Verify target server supports TLS

## Performance Issues

### Slow Performance

**Issue**: Low requests per second (RPS)
**Solutions**:
1. Check network connectivity
2. Use a server closer to the target
3. Increase concurrency (within system limits)
4. Reduce delays between requests

**Issue**: System resource exhaustion
**Solutions**:
1. Monitor system resources with `htop`, `iostat`, or `vmstat`
2. Reduce concurrency to match system capabilities
3. Close other applications that consume resources

## Proxy Issues

### Proxy Connection Failures

**Issue**: "Could not connect to proxy"
**Solutions**:
1. Verify proxy URL format: `http://proxy:port`
2. Check if proxy server is running
3. Verify proxy authentication if required

### Proxy Performance Issues

**Issue**: Slower performance when using proxy
**Solutions**:
1. Test proxy performance independently
2. Try a different proxy server
3. Consider proxy location (geographical distance)

## Authentication and Headers

### Header Issues

**Issue**: Requests being rejected due to headers
**Solutions**:
1. Verify header format: `Header-Name: value`
2. Check if target requires specific headers
3. Ensure no spaces around the colon

### Authorization Problems

**Issue**: Requests failing with 401/403 errors
**Solutions**:
1. Verify authorization tokens are valid
2. Check token expiration
3. Ensure proper header format for authentication

## Network Issues

### Rate Limiting

**Issue**: Target returns 429 (Too Many Requests) errors
**Solutions**:
1. Reduce request rate with `-w` (delay) option
2. Reduce concurrency with `-c` option
3. Implement exponential backoff if possible

### Firewall Blocking

**Issue**: Requests blocked by firewall
**Solutions**:
1. Check if target has IP-based blocking
2. Use proxy or VPN to change IP
3. Reduce request rate to avoid triggers

## System Configuration

### Resource Limits

Check system limits that might affect GLIROR:

```bash
# Check current file descriptor limit
ulimit -n

# Check network buffer sizes
cat /proc/sys/net/core/rmem_max
cat /proc/sys/net/core/wmem_max

# Check connection tracking limit
cat /proc/sys/net/netfilter/nf_conntrack_max
```

### Optimizing System for High Concurrency

```bash
# Increase network buffer sizes (as root)
echo 'net.core.rmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
echo 'net.core.wmem_max = 16777216' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p

# Increase connection tracking limit
echo 'net.netfilter.nf_conntrack_max = 131072' | sudo tee -a /etc/sysctl.conf
sudo sysctl -p
```

## Debugging Tips

### Enable Verbose Logging

While GLIROR doesn't have a built-in verbose mode, you can:
1. Redirect to log file: `gliror [options] 2>&1 | tee debug.log`
2. Use system tools to monitor: `strace -p $(pgrep gliror)` (if installed)

### Testing Connectivity

Before running tests, verify connectivity:
```bash
# Test connectivity
curl -I https://example.com

# Test with specific headers
curl -H "User-Agent: Test" https://example.com

# Test with proxy
curl -x http://proxy:port https://example.com
```

### Isolating Issues

To identify specific problems:
1. Start with minimal configuration
2. Add options one by one
3. Test with local server first if possible

## Platform-Specific Issues

### Windows Issues

- Use Git Bash or PowerShell for better compatibility
- Ensure OpenSSL is properly installed
- Consider WSL for full Linux compatibility

### macOS Issues

- May require Xcode command line tools: `xcode-select --install`
- Homebrew OpenSSL might need linking: `brew link openssl`

### Docker/Container Issues

If running in container:
- Ensure sufficient resources are allocated
- Check network configuration
- Verify DNS resolution works

## Prevention

### Before Running Tests

1. Start with low concurrency
2. Use shorter test durations initially
3. Have a monitoring tool ready
4. Ensure adequate system resources
5. Test on your own systems first

### During Tests

1. Monitor system resources
2. Watch for error patterns
3. Be ready to stop tests if needed
4. Log output if troubleshooting is needed

If you encounter an issue not covered here, consider:
- Checking the latest version of GLIROR
- Verifying your target is accessible
- Testing with a local server to isolate network issues
- Reviewing the system requirements and installation guide again