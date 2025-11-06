# Rustbuster
<p align="left">
  <img src="https://img.shields.io/badge/rust-1.70+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey.svg" alt="Platform">
</p>

```A fast, professional web enumeration tool written in Rust for security professionals and penetration testers.```

## Features

### Multiple Scanning Modes

* **Directory Enumeration** (`dir`) - Discover hidden directories and files
* **DNS Subdomain Discovery** (`dns`) - Find subdomains through brute-forcing
* **Virtual Host Enumeration** (`vhost`) - Identify virtual hosts on a server
* **Parameter Fuzzing** (`fuzz`) - Test for injection points and hidden parameters

### High Performance

* Concurrent scanning with configurable thread count
* Async I/O using Tokio for maximum throughput
* Efficient HTTP client with connection pooling
* Smart rate limiting and timeout handling

### Advanced Features

* **TUI Interface** - Real-time progress monitoring with scrollable results
* **Flexible Filtering** - Status codes, regex patterns, content length
* **Multiple Output Formats** - Plain text, JSON, CSV
* **Proxy Support** - HTTP, HTTPS, SOCKS4, SOCKS5
* **User Agent Rotation** - Evade basic detection
* **Custom Headers & Cookies** - Full request customization
* **Recursive Scanning** - Automatic directory traversal
* **Wildcard Detection** - Identify and handle wildcard responses

## Installation

### From Source

```bash
git clone https://github.com/wvbbb/rustbuster
cd rustbuster
cargo build --release
```

The binary will be available at `target/release/rustbuster`

### Using Cargo

```bash
cargo install --path .
```

## Quick Start

### Directory Enumeration

```bash
# Basic directory scan
rustbuster dir -u http://example.com -w wordlist.txt

# With file extensions
rustbuster dir -u http://example.com -w wordlist.txt -x php,html,txt

# Recursive scanning
rustbuster dir -u http://example.com -w wordlist.txt -R --depth 3

# Include backup file extensions
rustbuster dir -u http://example.com -w wordlist.txt --backup-extensions
```

### DNS Subdomain Discovery

```bash
# Basic subdomain enumeration
rustbuster dns -d example.com -w subdomains.txt

# Show IP addresses
rustbuster dns -d example.com -w subdomains.txt --show-ips

# Show CNAME records
rustbuster dns -d example.com -w subdomains.txt --show-cname
```

### Virtual Host Enumeration

```bash
# Basic vhost scan
rustbuster vhost -u http://example.com -w vhosts.txt

# With custom headers
rustbuster vhost -u http://example.com -w vhosts.txt -H "X-Forwarded-For: 127.0.0.1"
```

### Parameter Fuzzing

```bash
# Fuzz URL parameters (FUZZ keyword)
rustbuster fuzz -u http://example.com/page?id=FUZZ -w numbers.txt

# Fuzz path segments
rustbuster fuzz -u http://example.com/FUZZ/admin -w paths.txt
```

## Advanced Usage

### Performance Tuning

```bash
# Increase threads for faster scanning
rustbuster dir -u http://example.com -w wordlist.txt -t 50

# Adjust timeout for slow servers
rustbuster dir -u http://example.com -w wordlist.txt --timeout 15

# Add delay between requests (milliseconds)
rustbuster dir -u http://example.com -w wordlist.txt --delay 100
```

### Filtering Results

```bash
# Filter by status codes
rustbuster dir -u http://example.com -w wordlist.txt -s 200,301,302

# Exclude status codes
rustbuster dir -u http://example.com -w wordlist.txt -n 404,500

# Filter by regex pattern
rustbuster dir -u http://example.com -w wordlist.txt --filter-regex "error|not found"

# Match specific patterns
rustbuster dir -u http://example.com -w wordlist.txt --match-regex "admin|login"
```

### Authentication & Headers

```bash
# With authentication cookie
rustbuster dir -u http://example.com -w wordlist.txt -c "session=abc123"

# Custom headers
rustbuster dir -u http://example.com -w wordlist.txt -H "Authorization: Bearer token"

# Multiple headers
rustbuster dir -u http://example.com -w wordlist.txt \
  -H "Authorization: Bearer token" \
  -H "X-Custom-Header: value"
```

### Proxy Configuration

```bash
# HTTP proxy
rustbuster dir -u http://example.com -w wordlist.txt -p http://127.0.0.1:8080

# SOCKS5 proxy
rustbuster dir -u http://example.com -w wordlist.txt -p socks5://127.0.0.1:1080

# With Burp Suite
rustbuster dir -u http://example.com -w wordlist.txt -p http://127.0.0.1:8080 --no-tls-validation
```

### Output Options

```bash
# Save to file (plain text)
rustbuster dir -u http://example.com -w wordlist.txt -o results.txt

# JSON output
rustbuster dir -u http://example.com -w wordlist.txt -o results.json --output-format json

# CSV output
rustbuster dir -u http://example.com -w wordlist.txt -o results.csv --output-format csv

# Quiet mode (no banner)
rustbuster dir -u http://example.com -w wordlist.txt -q

# Verbose mode (detailed output)
rustbuster dir -u http://example.com -w wordlist.txt -v
```

### User Agent Rotation

```bash
# Single custom user agent
rustbuster dir -u http://example.com -w wordlist.txt -a "Mozilla/5.0 Custom"

# Rotate through user agents from file
rustbuster dir -u http://example.com -w wordlist.txt --user-agents-file ua.txt
```

## Wordlists

### Creating a Simple Wordlist

```bash
cat > wordlist.txt << EOF
admin
login
api
dashboard
config
backup
test
dev
staging
EOF
```

### Recommended Wordlists

For production use, consider these excellent wordlist collections:

* [SecLists](https://github.com/danielmiessler/SecLists) - Comprehensive security testing lists
* [FuzzDB](https://github.com/fuzzdb-project/fuzzdb) - Attack patterns and fuzzing data
* [Assetnote Wordlists](https://wordlists.assetnote.io/) - Curated for bug bounty hunting

## Performance Tips

1. **Thread Count**: Start with 10-20 threads and adjust based on target response

   * Fast servers: 50-100 threads
   * Slow/rate-limited servers: 5-10 threads

2. **Timeout Settings**:

   * Fast networks: 5-10 seconds
   * Slow/unstable connections: 15-30 seconds

3. **Status Code Filtering**: Reduce noise by excluding common codes

   ```bash
   -n 404,500,502,503
   ```

4. **Wordlist Optimization**: Use targeted wordlists for better results

   * Small wordlists (100-1000 words) for quick scans
   * Large wordlists (10k-100k words) for comprehensive enumeration

## TUI Interface

Rustbuster features a modern Terminal User Interface (TUI) for real-time monitoring:

* **Progress Bar**: Shows scan progress with percentage and ETA
* **Live Results**: Scrollable list of discovered resources
* **Statistics**: Real-time counters for scanned/found items
* **Keyboard Controls**:

  * ```Arrow keys / j/k```: Scroll results
  * ```Home/End or g/G```: Jump to top/bottom
  * ```PageUp/PageDown```: Fast scrolling
  * ```q: Quit```

Disable TUI with `--no-tui` flag for scripting or piping output.

## Examples

### Bug Bounty Hunting

```bash
# Comprehensive directory scan with common extensions
rustbuster dir -u https://target.com -w /usr/share/seclists/Discovery/Web-Content/common.txt \
  -x php,html,js,txt,xml,json -t 30 -o results.txt

# Subdomain enumeration
rustbuster dns -d target.com -w /usr/share/seclists/Discovery/DNS/subdomains-top1million-5000.txt \
  --show-ips -o subdomains.txt
```

### Penetration Testing

```bash
# Authenticated directory scan
rustbuster dir -u https://target.com/admin -w wordlist.txt \
  -c "PHPSESSID=abc123" -H "Authorization: Bearer token" \
  -s 200,301,302,403 -t 20

# Recursive scan with backup files
rustbuster dir -u https://target.com -w wordlist.txt \
  -R --depth 2 --backup-extensions -t 15
```

### API Testing

```bash
# API endpoint discovery
rustbuster dir -u https://api.target.com/v1 -w api-endpoints.txt \
  -H "Content-Type: application/json" -H "X-API-Key: key" \
  -s 200,201,400,401,403 --output-format json -o api-results.json
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_wordlist_from_file
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This tool is intended for legal security testing and research purposes only. Users are responsible for ensuring they have proper authorization before scanning any targets. The authors assume no liability for misuse or damage caused by this tool.

## Acknowledgments

* Inspired by [Gobuster](https://github.com/OJ/gobuster)
* Built with [Tokio](https://tokio.rs/) for async runtime
* TUI powered by [Ratatui](https://github.com/ratatui-org/ratatui)

## Support

If you encounter any issues or have questions:

* Open an issue on GitHub
* Check existing issues for solutions
* Read the documentation carefully
