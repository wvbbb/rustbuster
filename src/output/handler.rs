use crate::cli::CommonArgs;
use crate::core::http_client::ScanResult;
use colored::*;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use terminal_size::{Width, terminal_size};

#[derive(Clone)]
pub struct OutputHandler {
    output_file: Option<String>,
    output_format: String,
    quiet: bool,
    verbose: bool, // Added verbose field
    discovered_dirs: Arc<Mutex<Vec<String>>>,
    results_buffer: Arc<Mutex<Vec<ScanResult>>>,
}

impl OutputHandler {
    pub fn new(output_file: Option<String>, quiet: bool, output_format: String, verbose: bool) -> Self {
        OutputHandler {
            output_file,
            output_format,
            quiet,
            verbose, // Initialize verbose field
            discovered_dirs: Arc::new(Mutex::new(Vec::new())),
            results_buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_terminal_width() -> usize {
        if let Some((Width(w), _)) = terminal_size() {
            (w as usize).max(40) // Ensure minimum width of 40 for small terminals
        } else {
            80 // Default fallback width
        }
    }

    fn separator_line() -> String {
        let width = Self::get_terminal_width();
        "=".repeat(width.min(100)) // Cap at terminal width, min 40, max 100
    }

    /// Prints a banner with common configuration details
    pub fn print_banner_common(&self, args: &CommonArgs) {
        if self.quiet {
            return;
        }

        let separator = Self::separator_line();
        
        println!("{}", separator.bright_cyan());
        println!("{}", "Rustbuster v0.1.0".bright_cyan().bold());
        println!("{}", "Fast Web Directory Brute-Forcing Tool".bright_cyan());
        println!("{}", separator.bright_cyan());
        println!();
        println!("{} {}", "Wordlist:".bright_yellow(), 
            args.wordlist.as_deref().unwrap_or("None"));
        println!("{} {}", "Threads:".bright_yellow(), args.threads);
        println!("{} {}", "Timeout:".bright_yellow(), format!("{}s", args.timeout));
        
        if self.verbose {
            println!("{} Enabled", "Verbose Mode:".bright_yellow());
        }
        
        if args.delay.is_some() {
            println!("{} {}ms", "Delay:".bright_yellow(), args.delay.unwrap());
        }
        if args.user_agents_file.is_some() {
            println!("{} Enabled", "User-Agent Rotation:".bright_yellow());
        }
        if args.filter_regex.is_some() {
            println!("{} {}", "Filter Regex:".bright_yellow(), args.filter_regex.as_ref().unwrap());
        }
        if args.match_regex.is_some() {
            println!("{} {}", "Match Regex:".bright_yellow(), args.match_regex.as_ref().unwrap());
        }
        
        println!();
        println!("{}", separator.bright_cyan());
        println!();
    }

    /// Prints a scan result with enhanced information
    pub fn print_result(&self, result: &ScanResult, expanded: bool) {
        if self.quiet && !expanded {
            return;
        }

        if result.status_code == 301 || result.status_code == 302 || result.status_code == 200 {
            if result.url.ends_with('/') {
                if let Ok(mut dirs) = self.discovered_dirs.lock() {
                    dirs.push(result.url.clone());
                }
            }
        }

        if self.output_format != "plain" {
            if let Ok(mut buffer) = self.results_buffer.lock() {
                buffer.push(ScanResult {
                    url: result.url.clone(),
                    status_code: result.status_code,
                    content_length: result.content_length,
                    redirect_location: result.redirect_location.clone(),
                    body: None,
                    content_type: result.content_type.clone(),
                    server: result.server.clone(),
                    duration_ms: result.duration_ms,
                });
            }
        }

        let status_color = match result.status_code {
            200..=299 => "green",
            300..=399 => "yellow",
            400..=499 => "red",
            500..=599 => "magenta",
            _ => "white",
        };

        let mut output = format!(
            "{} [{} {}] [Size: {}]",
            result.url.bright_white(),
            result.status_code.to_string().color(status_color).bold(),
            result.status_text().color(status_color),
            result.content_length
        );

        if let Some(content_type) = &result.content_type {
            output.push_str(&format!(" [Type: {}]", content_type.bright_cyan()));
        }

        if let Some(server) = &result.server {
            output.push_str(&format!(" [Server: {}]", server.bright_magenta()));
        }

        if let Some(location) = &result.redirect_location {
            output.push_str(&format!(" -> {}", location.bright_blue()));
        }

        println!("{}", output);

        if self.output_format == "plain" {
            if let Some(file_path) = &self.output_file {
                let _ = self.write_plain_to_file(file_path, result);
            }
        }
    }

    fn write_plain_to_file(&self, file_path: &str, result: &ScanResult) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let line = if let Some(location) = &result.redirect_location {
            format!(
                "{} [{}] [{}] -> {}\n",
                result.url, result.status_code, result.content_length, location
            )
        } else {
            format!(
                "{} [{}] [{}]\n",
                result.url, result.status_code, result.content_length
            )
        };

        file.write_all(line.as_bytes())?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn finalize(&self) -> std::io::Result<()> {
        if let Some(file_path) = &self.output_file {
            if self.output_format == "json" {
                self.write_json_to_file(file_path)?;
            } else if self.output_format == "csv" {
                self.write_csv_to_file(file_path)?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn write_json_to_file(&self, file_path: &str) -> std::io::Result<()> {
        let results = self.results_buffer.lock().unwrap();
        let json_results: Vec<_> = results
            .iter()
            .map(|r| {
                json!({
                    "url": r.url,
                    "status_code": r.status_code,
                    "content_length": r.content_length,
                    "redirect_location": r.redirect_location,
                    "content_type": r.content_type,
                    "server": r.server,
                    "duration_ms": r.duration_ms,
                })
            })
            .collect();

        let json_output = serde_json::to_string_pretty(&json_results)?;
        std::fs::write(file_path, json_output)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn write_csv_to_file(&self, file_path: &str) -> std::io::Result<()> {
        let results = self.results_buffer.lock().unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;

        // Write CSV header
        writeln!(file, "URL,Status Code,Status Text,Content Length,Redirect Location,Content Type,Server,Duration (ms)")?;

        // Write results
        for result in results.iter() {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{}",
                result.url,
                result.status_code,
                result.status_text(),
                result.content_length,
                result.redirect_location.as_deref().unwrap_or(""),
                result.content_type.as_deref().unwrap_or(""),
                result.server.as_deref().unwrap_or(""),
                result.duration_ms
            )?;
        }

        Ok(())
    }

    pub fn print_summary(&self, total: usize, found: usize) {
        if self.quiet {
            return;
        }

        let separator = Self::separator_line();
        
        println!();
        println!("{}", separator.bright_cyan());
        println!(
            "{} Scanned: {}, Found: {}",
            "Summary:".bright_yellow().bold(),
            total,
            found
        );
        println!("{}", separator.bright_cyan());
    }

    #[allow(dead_code)]
    pub fn get_discovered_dirs(&self) -> Vec<String> {
        self.discovered_dirs.lock().unwrap().clone()
    }
}
