use crate::core::http_client::ScanResult;
use anyhow::Result;
use chrono::Utc;
use std::fs;

/// Generates HTML reports from scan results
#[allow(dead_code)]
pub struct ReportGenerator {
    results: Vec<ScanResult>,
    target: String,
    scan_duration: u64,
}

impl ReportGenerator {
    /// Creates a new report generator for a target

    pub fn new(target: String) -> Self {
        ReportGenerator {
            results: Vec::new(),
            target,
            scan_duration: 0,
        }
    }

    /// Adds a scan result to the report

    pub fn add_result(&mut self, result: ScanResult) {
        self.results.push(result);
    }

    /// Sets the total scan duration in seconds

    pub fn set_duration(&mut self, duration: u64) {
        self.scan_duration = duration;
    }

    /// Generates and saves the HTML report to a file

    pub fn generate_html(&self, output_path: &str) -> Result<()> {
        let html = self.build_html();
        fs::write(output_path, html)?;
        println!("[+] HTML report generated: {}", output_path);
        Ok(())
    }

    /// Builds the HTML content for the report

    fn build_html(&self) -> String {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        let mut status_counts = std::collections::HashMap::new();
        for result in &self.results {
            *status_counts.entry(result.status_code).or_insert(0) += 1;
        }

        let mut results_html = String::new();
        for result in &self.results {
            let status_class = match result.status_code {
                200..=299 => "success",
                300..=399 => "redirect",
                400..=499 => "client-error",
                500..=599 => "server-error",
                _ => "other",
            };

            results_html.push_str(&format!(
                r#"<tr class="{}">
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>"#,
                status_class,
                html_escape(&result.url),
                result.status_code,
                result.content_length,
                result.redirect_location.as_deref().unwrap_or("-")
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rustbuster Scan Report</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; background: #0a0e27; color: #e0e0e0; padding: 20px; }}
        .container {{ max-width: 1400px; margin: 0 auto; }}
        header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 30px; border-radius: 10px; margin-bottom: 30px; box-shadow: 0 4px 6px rgba(0,0,0,0.3); }}
        h1 {{ color: white; font-size: 2.5em; margin-bottom: 10px; }}
        .subtitle {{ color: #f0f0f0; font-size: 1.1em; }}
        .stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .stat-card {{ background: #1a1f3a; padding: 20px; border-radius: 8px; border-left: 4px solid #667eea; }}
        .stat-label {{ color: #888; font-size: 0.9em; margin-bottom: 5px; }}
        .stat-value {{ color: #fff; font-size: 2em; font-weight: bold; }}
        .results-section {{ background: #1a1f3a; padding: 20px; border-radius: 8px; }}
        table {{ width: 100%; border-collapse: collapse; }}
        th {{ background: #2a2f4a; color: #fff; padding: 15px; text-align: left; font-weight: 600; }}
        td {{ padding: 12px 15px; border-bottom: 1px solid #2a2f4a; }}
        tr:hover {{ background: #252a45; }}
        .success {{ background: rgba(76, 175, 80, 0.1); }}
        .redirect {{ background: rgba(255, 193, 7, 0.1); }}
        .client-error {{ background: rgba(244, 67, 54, 0.1); }}
        .server-error {{ background: rgba(156, 39, 176, 0.1); }}
        .footer {{ text-align: center; margin-top: 30px; color: #666; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🦀 Rustbuster Scan Report</h1>
            <div class="subtitle">Fast Web Directory Brute-Forcing Tool</div>
        </header>

        <div class="stats">
            <div class="stat-card">
                <div class="stat-label">Target</div>
                <div class="stat-value" style="font-size: 1.2em;">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Total Findings</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Scan Duration</div>
                <div class="stat-value">{}s</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Scan Time</div>
                <div class="stat-value" style="font-size: 1em;">{}</div>
            </div>
        </div>

        <div class="results-section">
            <h2 style="margin-bottom: 20px; color: #667eea;">Discovered Resources</h2>
            <table>
                <thead>
                    <tr>
                        <th>URL</th>
                        <th>Status</th>
                        <th>Size</th>
                        <th>Redirect</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
        </div>

        <div class="footer">
            <p>Generated by Rustbuster v0.1.0 | {}</p>
        </div>
    </div>
</body>
</html>"#,
            html_escape(&self.target),
            self.results.len(),
            self.scan_duration,
            timestamp,
            results_html,
            timestamp
        )
    }
}

/// Escapes HTML special characters
#[allow(dead_code)]
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
