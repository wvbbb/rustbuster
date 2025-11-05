use std::process::Command;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Rustbuster"));
    assert!(stdout.contains("Usage"));
    assert!(stdout.contains("dir") || stdout.contains("dns") || stdout.contains("vhost") || stdout.contains("fuzz"));
}

#[test]
fn test_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.to_lowercase().contains("rustbuster"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_dir_mode_requires_url() {
    let output = Command::new("cargo")
        .args(&["run", "--", "dir", "-w", "wordlist.txt"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("USAGE") || stderr.contains("Usage"));
}

#[test]
fn test_dns_mode_requires_domain() {
    let output = Command::new("cargo")
        .args(&["run", "--", "dns", "-w", "wordlist.txt"])
        .output()
        .expect("Failed to execute command");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("USAGE") || stderr.contains("Usage"));
}
