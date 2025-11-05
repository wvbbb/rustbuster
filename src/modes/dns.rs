use crate::cli::DnsArgs;
use crate::core::Wordlist;
use crate::output::{tui, OutputHandler};
use crate::output::tui::{TuiMessage, TuiResult};
use anyhow::Result;
use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;
use tokio::sync::mpsc;

pub async fn run(args: DnsArgs) -> Result<()> {
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

    // Generate subdomains to test
    let subdomains: Vec<String> = wordlist
        .words
        .iter()
        .map(|word| format!("{}.{}", word, args.domain))
        .collect();

    let total = subdomains.len();
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

    // Create DNS resolver
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    );

    // Process subdomains concurrently
    stream::iter(subdomains)
        .map(|subdomain| {
            let resolver = &resolver;
            let found = Arc::clone(&found_clone);
            let progress = &progress;
            let expanded = args.common.expanded;
            let show_ips = args.show_ips;
            let quiet = args.common.quiet;

            async move {
                if let Some(pb) = progress {
                    pb.inc(1);
                }

                let start = Instant::now();
                match resolver.lookup_ip(&subdomain).await {
                    Ok(response) => {
                        let _duration_ms = start.elapsed().as_millis() as u64;
                        
                        let ips: Vec<String> = response
                            .iter()
                            .map(|ip| ip.to_string())
                            .collect();

                        if !ips.is_empty() {
                            found.fetch_add(1, Ordering::SeqCst);
                            if !quiet {
                                if show_ips {
                                    println!(
                                        "{} -> {}",
                                        subdomain.bright_white(),
                                        ips.join(", ").bright_green()
                                    );
                                } else {
                                    println!("{}", subdomain.bright_white());
                                }
                            }
                        }
                    }
                    Err(_) => {
                        if expanded {
                            eprintln!("No DNS record for: {}", subdomain);
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

async fn run_with_tui(args: DnsArgs) -> Result<()> {
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;

    let subdomains: Vec<String> = wordlist
        .words
        .iter()
        .map(|word| format!("{}.{}", word, args.domain))
        .collect();

    let total = subdomains.len();
    let threads = args.common.threads;
    let domain = args.domain.clone();
    
    tui::run_tui_mode(
        "dns".to_string(),
        domain.clone(),
        wordlist_path.clone(),
        threads,
        total,
        args.common.output.clone(),
        args.common.output_format.clone(),
        move |tx| async move {
            scan_dns_with_tui(subdomains, threads, tx).await
        },
    ).await
}

async fn scan_dns_with_tui(
    subdomains: Vec<String>,
    threads: usize,
    tx: mpsc::Sender<TuiMessage>,
) -> Result<()> {
    let resolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default(),
    );

    stream::iter(subdomains)
        .map(|subdomain| {
            let resolver = &resolver;
            let tx = tx.clone();

            async move {
                let _ = tx.send(TuiMessage::Scanned).await;

                let start = Instant::now();
                match resolver.lookup_ip(&subdomain).await {
                    Ok(response) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        
                        let ips: Vec<String> = response
                            .iter()
                            .map(|ip| ip.to_string())
                            .collect();

                        if !ips.is_empty() {
                            let result = TuiResult {
                                url: subdomain,
                                status_code: 200,
                                content_length: 0,
                                redirect_location: Some(ips.join(", ")),
                                content_type: None,
                                server: None,
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
