use ansi_term::Style;

pub fn print_arguments_help() {
    println!("\n{}", Style::new().bold().paint("Rustbuster - ALL ARGUMENTS"));
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    print_section("CORE OPTIONS", vec![
        ("-w, --wordlist <FILE>", "Path to wordlist file (one entry per line)"),
        ("-t, --threads <NUM>", "Number of concurrent threads (default: 10)"),
        ("--timeout <SECS>", "HTTP request timeout in seconds (default: 10)"),
    ]);

    print_section("STATUS CODE FILTERING", vec![
        ("-s, --status-codes <CODES>", "Positive status codes to report (default: 200,204,301,302,307,401,403)"),
        ("-n, --negative-status-codes <CODES>", "Negative status codes to exclude"),
    ]);

    print_section("HTTP OPTIONS", vec![
        ("-r, --follow-redirects", "Follow HTTP redirects (3xx responses)"),
        ("-a, --user-agent <STRING>", "User-Agent string (default: rustbuster/0.1.0)"),
        ("--user-agents-file <FILE>", "File with multiple User-Agents for rotation"),
        ("--method <METHOD>", "HTTP method (default: GET)"),
        ("-c, --cookies <STRING>", "Cookies to send (format: \"name1=value1; name2=value2\")"),
        ("-H, --headers <HEADER>", "Custom HTTP headers (can be used multiple times)"),
    ]);

    print_section("PROXY & TLS OPTIONS", vec![
        ("-p, --proxy <URL>", "Proxy URL (HTTP/HTTPS/SOCKS4/SOCKS5)"),
        ("--no-tls-validation", "Skip TLS certificate validation"),
    ]);

    print_section("OUTPUT OPTIONS", vec![
        ("-e, --expanded", "Show all responses including negative status codes"),
        ("-q, --quiet", "Suppress banner and reduce output verbosity"),
        ("-v, --verbose", "Show detailed errors and debug output"),
        ("--no-progress", "Disable progress bar display"),
        ("-o, --output <FILE>", "Save results to output file"),
        ("--output-format <FORMAT>", "Output format: plain, json, csv (default: plain)"),
    ]);

    print_section("FILTERING OPTIONS", vec![
        ("--wildcard", "Force continue on wildcard responses"),
        ("--filter-regex <REGEX>", "Filter responses by regex pattern (exclude matches)"),
        ("--match-regex <REGEX>", "Match responses by regex pattern (only show matches)"),
        ("--filter-size <SIZES>", "Filter responses by content length (comma-separated)"),
    ]);

    print_section("RATE LIMITING", vec![
        ("--delay <MS>", "Delay between requests in milliseconds"),
    ]);

    print_section("SESSION MANAGEMENT", vec![
        ("--save-session <NAME>", "Save scan session to resume later"),
        ("--resume-session <NAME>", "Resume a previously saved session"),
    ]);

    print_section("ADVANCED FEATURES", vec![
        ("--smart-404", "Enable smart 404 detection"),
        ("--targets <FILE>", "File with multiple target URLs/domains"),
        ("--report <FILE>", "Generate professional HTML report"),
        ("--similarity-threshold <FLOAT>", "Response similarity detection (0.0-1.0)"),
    ]);

    print_section("MODE-SPECIFIC OPTIONS", vec![
        ("", &format!("{}", Style::new().bold().paint("DIR MODE:"))),
        ("  -u, --url <URL>", "Target base URL to scan"),
        ("  -x, --extensions <EXTS>", "File extensions (comma-separated)"),
        ("  -R, --recursive", "Enable recursive scanning"),
        ("  --depth <NUM>", "Maximum recursion depth (default: 3)"),
        ("  --backup-extensions", "Try common backup file extensions"),
        ("", ""),
        ("", &format!("{}", Style::new().bold().paint("DNS MODE:"))),
        ("  -d, --domain <DOMAIN>", "Target domain to enumerate"),
        ("  --show-cname", "Display CNAME records"),
        ("  --show-ips", "Display resolved IP addresses"),
        ("", ""),
        ("", &format!("{}", Style::new().bold().paint("VHOST MODE:"))),
        ("  -u, --url <URL>", "Target URL to test virtual hosts"),
        ("", ""),
        ("", &format!("{}", Style::new().bold().paint("FUZZ MODE:"))),
        ("  -u, --url <URL>", "Target URL with FUZZ keyword(s)"),
        ("  -x, --extensions <EXTS>", "File extensions (comma-separated)"),
    ]);

    println!("TIP: Use 'rustbuster <MODE> --help' for mode-specific help");
    println!("     Use 'rustbuster --examples' to see usage examples");
    println!("     Use 'rustbuster --info' for additional information\n");
}

pub fn print_examples() {
    println!("\n{}", Style::new().bold().paint("rustbuster - USAGE EXAMPLES"));
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    print_example_section("DIRECTORY ENUMERATION", vec![
        ("Basic scan", "rustbuster dir -u http://example.com -w wordlist.txt"),
        ("With extensions", "rustbuster dir -u http://example.com -w wordlist.txt -x php,html,txt"),
        ("Recursive scan", "rustbuster dir -u http://example.com -w wordlist.txt -R --depth 3"),
        ("Find backups", "rustbuster dir -u http://example.com -w wordlist.txt --backup-extensions"),
        ("With auth", "rustbuster dir -u http://example.com -w wordlist.txt -H \"Authorization: Bearer TOKEN\""),
        ("Through proxy", "rustbuster dir -u http://example.com -w wordlist.txt -p http://127.0.0.1:8080"),
    ]);

    print_example_section("DNS SUBDOMAIN ENUMERATION", vec![
        ("Basic scan", "rustbuster dns -d example.com -w subdomains.txt"),
        ("Show IPs", "rustbuster dns -d example.com -w subdomains.txt --show-ips"),
        ("Show all info", "rustbuster dns -d example.com -w subdomains.txt --show-ips --show-cname"),
    ]);

    print_example_section("VIRTUAL HOST DISCOVERY", vec![
        ("Basic scan", "rustbuster vhost -u http://example.com -w vhosts.txt"),
        ("Scan IP", "rustbuster vhost -u http://192.168.1.1 -w vhosts.txt"),
        ("Custom host", "rustbuster vhost -u http://192.168.1.1 -w vhosts.txt -H \"Host: example.com\""),
    ]);

    print_example_section("FUZZING MODE", vec![
        ("Basic fuzz", "rustbuster fuzz -u http://example.com/FUZZ -w wordlist.txt"),
        ("API fuzzing", "rustbuster fuzz -u http://example.com/api/FUZZ -w params.txt"),
        ("With extensions", "rustbuster fuzz -u http://example.com/FUZZ -w wordlist.txt -x json,xml"),
        ("Multiple FUZZ", "rustbuster fuzz -u http://example.com/FUZZ/FUZZ -w wordlist.txt"),
    ]);

    print_example_section("PROXY USAGE", vec![
        ("Burp Suite", "rustbuster dir -u http://example.com -w wordlist.txt -p http://127.0.0.1:8080"),
        ("OWASP ZAP", "rustbuster dir -u http://example.com -w wordlist.txt -p http://127.0.0.1:8081"),
        ("Tor/SOCKS5", "rustbuster dir -u http://example.com -w wordlist.txt -p socks5://127.0.0.1:9050"),
        ("With auth", "rustbuster dir -u http://example.com -w wordlist.txt -p http://user:pass@proxy.com:8080"),
    ]);

    print_example_section("SESSION MANAGEMENT", vec![
        ("Save session", "rustbuster dir -u http://example.com -w wordlist.txt --save-session scan1"),
        ("Resume session", "rustbuster dir --resume-session scan1"),
    ]);

    print_example_section("ADVANCED FEATURES", vec![
        ("Multi-target", "rustbuster dir -w wordlist.txt --targets targets.txt"),
        ("Smart 404", "rustbuster dir -u http://example.com -w wordlist.txt --smart-404"),
        ("HTML report", "rustbuster dir -u http://example.com -w wordlist.txt --report report.html"),
        ("Rate limiting", "rustbuster dir -u http://example.com -w wordlist.txt --delay 100"),
        ("User-Agent rotation", "rustbuster dir -u http://example.com -w wordlist.txt --user-agents-file ua.txt"),
        ("Response filtering", "rustbuster dir -u http://example.com -w wordlist.txt --filter-size 1234 --match-regex \"admin\""),
    ]);

    print_example_section("OUTPUT FORMATS", vec![
        ("JSON output", "rustbuster dir -u http://example.com -w wordlist.txt -o results.json --output-format json"),
        ("CSV output", "rustbuster dir -u http://example.com -w wordlist.txt -o results.csv --output-format csv"),
        ("Quiet mode", "rustbuster dir -u http://example.com -w wordlist.txt -q -o results.txt"),
        ("Verbose mode", "rustbuster dir -u http://example.com -w wordlist.txt -v"),
    ]);

    println!("For more information: https://github.com/rustbuster/rustbuster");
    println!("Report bugs: https://github.com/rustbuster/rustbuster/issues\n");
}

pub fn print_info() {
    println!("\n{}", Style::new().bold().paint("rustbuster - ADDITIONAL INFORMATION"));
    println!("═══════════════════════════════════════════════════════════════════════════════\n");

    print_section("ABOUT", vec![
        ("", "rustbuster is a fast, professional web enumeration tool written in Rust."),
        ("", "It supports directory/file enumeration, DNS subdomain discovery, virtual"),
        ("", "host enumeration, and fuzzing with advanced features like session management,"),
        ("", "smart 404 detection, and response similarity analysis."),
    ]);

    print_section("FEATURES", vec![
        ("✓", "Multiple scanning modes (dir, dns, vhost, fuzz)"),
        ("✓", "Concurrent scanning with configurable threads"),
        ("✓", "Proxy support (HTTP/HTTPS/SOCKS4/SOCKS5)"),
        ("✓", "Session management (save/resume scans)"),
        ("✓", "Smart 404 detection and wildcard handling"),
        ("✓", "Response filtering and similarity detection"),
        ("✓", "Multiple output formats (plain, JSON, CSV)"),
        ("✓", "HTML report generation"),
        ("✓", "User-Agent rotation"),
        ("✓", "Rate limiting and delay options"),
        ("✓", "Recursive directory scanning"),
        ("✓", "Custom headers and authentication"),
    ]);

    print_section("WORDLISTS", vec![
        ("", "rustbuster works with any text-based wordlist (one entry per line)."),
        ("", "Popular wordlist collections:"),
        ("", "  • SecLists: https://github.com/danielmiessler/SecLists"),
        ("", "  • FuzzDB: https://github.com/fuzzdb-project/fuzzdb"),
        ("", "  • Assetnote: https://wordlists.assetnote.io/"),
    ]);

    print_section("PERFORMANCE TIPS", vec![
        ("", "• Start with 10 threads and increase if needed"),
        ("", "• Use --delay to avoid rate limiting/WAF blocks"),
        ("", "• Enable --smart-404 for sites with custom error pages"),
        ("", "• Use --filter-size to exclude common response sizes"),
        ("", "• Save sessions for long scans with --save-session"),
        ("", "• Use --quiet mode for cleaner output in scripts"),
    ]);

    print_section("COMMON USE CASES", vec![
        ("Web App Testing", "Find hidden admin panels, backup files, and sensitive directories"),
        ("Bug Bounty", "Discover subdomains, virtual hosts, and API endpoints"),
        ("Penetration Testing", "Enumerate web infrastructure and identify attack surface"),
        ("Security Audits", "Map web application structure and identify misconfigurations"),
    ]);

    print_section("SUPPORT", vec![
        ("GitHub", "https://github.com/rustbuster/rustbuster"),
        ("Issues", "https://github.com/rustbuster/rustbuster/issues"),
        ("Documentation", "https://github.com/rustbuster/rustbuster/wiki"),
    ]);

    println!();
}

fn print_section(title: &str, items: Vec<(&str, &str)>) {
    println!("{}", Style::new().bold().paint(title));
    println!("───────────────────────────────────────────────────────────────────────────────");
    for (flag, desc) in items {
        if flag.is_empty() {
            if desc.is_empty() {
                println!();
            } else {
                println!("  {}", desc);
            }
        } else {
            println!("  {:<35} {}", flag, desc);
        }
    }
    println!();
}

fn print_example_section(title: &str, examples: Vec<(&str, &str)>) {
    println!("{}", Style::new().bold().paint(title));
    println!("───────────────────────────────────────────────────────────────────────────────");
    for (desc, cmd) in examples {
        println!("  → {}", desc);
        println!("    {}", cmd);
        println!();
    }
}
