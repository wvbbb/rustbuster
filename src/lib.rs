// Main crate for Rustbuster, wiring submodules together

// CLI argument parsing and subcommands
pub mod cli;
// Core scanning, HTTP client, and wordlist utils
pub mod core;
// Scan modes: dir, dns, vhost, fuzz
pub mod modes;
// Output handling and TUI
pub mod output;
