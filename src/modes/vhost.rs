use crate::cli::VhostArgs;
use crate::core::{HttpClient, Wordlist};
use crate::output::{tui, OutputHandler};
use crate::output::tui::{TuiMessage, TuiResult};
use anyhow::Result;
use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

pub async fn run(args: VhostArgs) -> Result<()> {
    if !args.common.no_tui {
        return run_with_tui(args).await;
    }

    let output = OutputHandler::new(
        args.common.output.clone(),
        args.common.quiet,
        args.common.output_format.clone(),
        args.common.verbose,
    );
    output.print_banner_common(&args.common);

    // Load wordlist
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    let base_domain = args.url.trim_start_matches("http://").trim_start_matches("https://");

    // Generate vhosts to test
    let vhosts: Vec<String> = wordlist
        .words
        .iter()
        .map(|word| format!("{}.{}", word, base_domain))
        .collect();

    let total = vhosts.len();
    let found = Arc::new(AtomicUsize::new(0));
    let found_clone = Arc::clone(&found);

    // Setup progress bar
    let progress = if !args.common.no_progress && !args.common.quiet {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        Some(pb)
    } else {
        None
    };

    // Create HTTP client
    let client = HttpClient::new_from_common(&args.common)?;
    let status_codes = args.common.get_status_codes();
    let negative_codes = args.common.get_negative_status_codes();
    
    let default_status_codes = if status_codes.is_empty() && negative_codes.is_empty() {
        (200..300).collect::<Vec<u16>>()
    } else {
        status_codes.clone()
    };

    // Parse headers
    let headers: Vec<(String, String)> = args
        .common
        .headers
        .iter()
        .filter_map(|h| {
            let parts: Vec<&str> = h.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect();

    let cookies = args.common.cookies.as_deref();

    // Process vhosts concurrently
    stream::iter(vhosts)
        .map(|vhost| {
            let client = &client;
            let url = &args.url;
            let method = &args.common.method;
            let mut vhost_headers = headers.clone();
            let found = Arc::clone(&found_clone);
            let progress = &progress;
            let expanded = args.common.expanded;
            let status_codes = default_status_codes.clone();
            let negative_codes = negative_codes.clone();
            let quiet = args.common.quiet;

            async move {
                if let Some(pb) = progress {
                    pb.inc(1);
                }

                // Add Host header for vhost
                vhost_headers.push(("Host".to_string(), vhost.clone()));

                let start = Instant::now();
                match client.request(url, method, &vhost_headers, cookies).await {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        
                        let status = response.status().as_u16();
                        let content_length = response.content_length().unwrap_or(0);

                        let should_display = if !negative_codes.is_empty() {
                            !negative_codes.contains(&status)
                        } else if !status_codes.is_empty() {
                            status_codes.contains(&status)
                        } else {
                            // If no filters specified, show successful responses
                            (200..300).contains(&status)
                        };

                        if should_display || expanded {
                            found.fetch_add(1, Ordering::SeqCst);
                            
                            if !quiet {
                                let status_color = match status {
                                    200..=299 => "green",
                                    300..=399 => "yellow",
                                    400..=499 => "red",
                                    500..=599 => "magenta",
                                    _ => "white",
                                };

                                println!(
                                    "{} (Status: {}) [Size: {}] [Duration: {} ms]",
                                    vhost.bright_white(),
                                    status.to_string().color(status_color).bold(),
                                    content_length,
                                    duration_ms
                                );
                            }
                        }
                    }
                    Err(_) => {
                        if expanded {
                            eprintln!("Error testing vhost: {}", vhost);
                        }
                    }
                }
            }
        })
        .buffer_unordered(args.common.threads)
        .collect::<Vec<_>>()
        .await;

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    let found_count = found.load(Ordering::SeqCst);
    output.print_summary(total, found_count);

    Ok(())
}

async fn run_with_tui(args: VhostArgs) -> Result<()> {
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    let base_domain = args.url.trim_start_matches("http://").trim_start_matches("https://");

    let vhosts: Vec<String> = wordlist
        .words
        .iter()
        .map(|word| format!("{}.{}", word, base_domain))
        .collect();

    let total = vhosts.len();
    let client = HttpClient::new_from_common(&args.common)?;
    let url = args.url.clone();
    let method = args.common.method.clone();
    let threads = args.common.threads;
    
    let headers: Vec<(String, String)> = args
        .common
        .headers
        .iter()
        .filter_map(|h| {
            let parts: Vec<&str> = h.splitn(2, ':').collect();
            if parts.len() == 2 {
                Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
            } else {
                None
            }
        })
        .collect();

    let status_codes = args.common.get_status_codes();
    let negative_codes = args.common.get_negative_status_codes();
    
    tui::run_tui_mode(
        "vhost".to_string(),
        url.clone(),
        wordlist_path.clone(),
        threads,
        total,
        args.common.output.clone(),
        args.common.output_format.clone(),
        move |tx| async move {
            scan_vhost_with_tui(vhosts, client, url, method, headers, status_codes, negative_codes, threads, tx).await
        },
    ).await
}

async fn scan_vhost_with_tui(
    vhosts: Vec<String>,
    client: HttpClient,
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    status_codes: Vec<u16>,
    negative_codes: Vec<u16>,
    threads: usize,
    tx: mpsc::Sender<TuiMessage>,
) -> Result<()> {
    let default_status_codes = if status_codes.is_empty() && negative_codes.is_empty() {
        (200..300).collect::<Vec<u16>>()
    } else {
        status_codes.clone()
    };

    stream::iter(vhosts)
        .map(|vhost| {
            let client = &client;
            let url = &url;
            let method = &method;
            let mut vhost_headers = headers.clone();
            let tx = tx.clone();
            let status_codes = default_status_codes.clone();
            let negative_codes = negative_codes.clone();

            async move {
                let _ = tx.send(TuiMessage::Scanned).await;

                vhost_headers.push(("Host".to_string(), vhost.clone()));

                let start = Instant::now();
                match client.request(url, method, &vhost_headers, None).await {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        
                        let status = response.status().as_u16();
                        let content_length = response.content_length().unwrap_or(0);

                        let should_display = if !negative_codes.is_empty() {
                            !negative_codes.contains(&status)
                        } else if !status_codes.is_empty() {
                            status_codes.contains(&status)
                        } else {
                            (200..300).contains(&status)
                        };

                        if should_display {
                            let content_type = response
                                .headers()
                                .get("content-type")
                                .and_then(|v| v.to_str().ok())
                                .map(|s| s.split(';').next().unwrap_or(s).trim().to_string());
                            
                            let server = response
                                .headers()
                                .get("server")
                                .and_then(|v| v.to_str().ok())
                                .map(|s| s.to_string());

                            let result = TuiResult {
                                url: vhost,
                                status_code: status,
                                content_length,
                                redirect_location: None,
                                content_type,
                                server,
                                duration_ms,
                            };
                            let _ = tx.send(TuiMessage::Result(result)).await;
                        }
                    }
                    Err(_) => {
                        let _ = tx.send(TuiMessage::Error).await;
                    }
                }
            }
        })
        .buffer_unordered(threads)
        .collect::<Vec<_>>()
        .await;

    let _ = tx.send(TuiMessage::Done).await;
    Ok(())
}
