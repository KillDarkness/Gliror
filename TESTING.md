# GLIROR Testing Examples

## Local Testing Setup

For testing purposes, you can run a simple local HTTP server:

### Using Python (Python 3)
```bash
# Start a simple HTTP server on port 8000
python3 -m http.server 8000
```

### Using Node.js
```bash
# Install a simple static server
npm install -g http-server

# Start server on port 8000
http-server -p 8000
```

### Using Node.js Express (for POST/PUT testing)
```bash
# Create a simple server that handles different methods
npx express-generator test-server
cd test-server
npm install
npm start
```

## Running GLIROR Against Local Server

1. Start your local server first
2. Run GLIROR against it:

```bash
# Basic GET request test
cargo run --release -- --url http://localhost:8000 --time 10

# Unlimited attack (Ctrl+C to stop)
cargo run --release -- --url http://localhost:8000 --time 0

# POST request with data
cargo run --release -- --url http://localhost:8000/api --method POST --data '{"test": "data"}' --time 30

# With custom headers
cargo run --release -- --url http://localhost:8000 --header "User-Agent: TestBot/1.0" --time 20

# With custom concurrency
cargo run --release -- --url http://localhost:8000 --concurrent 50 --time 30

# With delay between requests (100ms delay)
cargo run --release -- --url http://localhost:8000 --delay 100 --time 30
```

## Example Output

```
  ▄████  ██▓     ██▓▄▄▄█████▓ ▒█████   ██▀███  
 ██▒ ▀█▒▓██▒    ▓██▒▓  ██▒ ▓▒▒██▒  ██▒▓██ ▒ ██▒
▒██░▄▄▄░▒██░    ▒██▒▒ ▓██░ ▒░▒██░  ██▒▓██ ░▄█ ▒
░▓█  ██▓▒██░    ░██░░ ▓██▓ ░ ▒██   ██░▒██▀▀█▄  
░▒▓███▀▒░██████▒░██░  ▒██▒ ░ ░ ████▓▒░░██▓ ▒██▒
 ░▒   ▒ ░ ▒░▓  ░░▓    ▒ ░░   ░ ▒░▒░▒░ ░ ▒▓ ░▒▓░
  ░   ░ ░ ░ ▒  ░ ▒ ░    ░      ░ ▒ ▒░   ░▒ ░ ▒░
░ ░   ░   ░ ░    ▒ ░  ░      ░ ░ ░ ▒    ░░   ░ 
      ░     ░  ░ ░             ░ ░     ░     

INFO Starting attack on: http://localhost:8000
INFO Method: GET
INFO Duration: 10 seconds
INFO Concurrent requests: 20
STATUS: Sent: 2500, Success: 2480, Errors: 20, RPS: 250.0, Avg: 15.2ms
```

## Performance Settings

- **Time = 0**: Fast mode, 100 concurrent requests
- **Time > 0**: Medium mode, 20 concurrent requests
- **Custom**: Use --concurrent flag to set your own number
- **HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD
- **Request Data**: Use --data to send payloads
- **Headers**: Use --header to add custom headers
- **Proxy**: Use --proxy to route through a proxy server
- **Delay**: Use --delay to add milliseconds between requests
- Timeout set to 10 seconds per request

## Monitoring

GLIROR will alert you in the following situations:

- If error rate exceeds 10%
- If individual requests take more than 2 seconds
- If average response time exceeds 1500ms
- Real-time requests per second (RPS) calculation
- Success/failure counts with percentages

## Important Notes

- This tool is intended for educational and authorized testing only
- Always obtain proper authorization before testing systems you don't own
- High concurrency settings may impact your own system's performance
- Use proxies to obfuscate your real IP address if needed
- Monitor your own system's resource usage during tests