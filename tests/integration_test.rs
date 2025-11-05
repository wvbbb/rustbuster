//! Integration tests for Rustbuster scanning modes

use std::process::Command;

#[test]
fn test_help_command() {
    // Run the binary with --help
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    // The command should exit successfully
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // clap 4.x outputs "Usage" with capital U
    assert!(stdout.contains("Rustbuster"));
    assert!(stdout.contains("Usage"));
    // Optionally check that the commands are listed
    assert!(stdout.contains("dir") || stdout.contains("dns") || stdout.contains("vhost") || stdout.contains("fuzz"));
}

#[test]
fn test_version_command() {
    // Run the binary with --version
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // clap 4.x prints the version in the format: rustbuster 0.1.0
    assert!(stdout.to_lowercase().contains("rustbuster"));
    assert!(stdout.contains("0.1.0")); // match the version in Cargo.toml
}

#[test]
fn test_dir_mode_requires_url() {
    // Missing the required URL argument
    let output = Command::new("cargo")
        .args(&["run", "--", "dir", "-w", "wordlist.txt"])
        .output()
        .expect("Failed to execute command");

    // Should fail because URL is required
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("USAGE") || stderr.contains("Usage"));
}

#[test]
fn test_dns_mode_requires_domain() {
    // Missing the required domain argument
    let output = Command::new("cargo")
        .args(&["run", "--", "dns", "-w", "wordlist.txt"])
        .output()
        .expect("Failed to execute command");

    // Should fail because domain is required
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("USAGE") || stderr.contains("Usage"));
}
