use crate::cli::CommonArgs;
use crate::core::http_client::{HttpClient, ScanResult};
use crate::output::handler::OutputHandler;
use crate::output::tui::{TuiMessage, TuiResult};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

pub struct Scanner {
    client: HttpClient,
    output: OutputHandler,
    threads: usize,
    discovered_dirs: Vec<String>,
}

impl Scanner {
    pub fn new_from_common(common: CommonArgs) -> Result<Self> {
        let client = HttpClient::new_from_common(&common)?;

        let output = OutputHandler::new(
            common.output.clone(),
            common.quiet,
            common.output_format.clone(),
            common.verbose,
        );

        Ok(Self {
            client,
            output,
            threads: common.threads,
            discovered_dirs: Vec::new(),
        })
    }

    pub async fn scan_urls(&mut self, urls: Vec<String>) -> Result<()> {
        let client = Arc::new(self.client.clone());
        let output = Arc::new(self.output.clone());

        stream::iter(urls)
            .map(|url| {
                let client = Arc::clone(&client);
                let output = Arc::clone(&output);
                async move {
                    let start = Instant::now();
                    match client.request(&url, "GET", &[], None).await {
                        Ok(response) => {
                            let duration_ms = start.elapsed().as_millis() as u64;
                            let result = ScanResult::from_response(url.clone(), &response, duration_ms);

                            if result.status_code == 301 || result.status_code == 302 {
                                // Note: Can't modify self.discovered_dirs from here due to Arc
                            }

                            output.print_result(&result, false);
                        }
                        Err(_) => {
                            // Error handling - could send to output if needed
                        }
                    }
                }
            })
            .buffer_unordered(self.threads)
            .collect::<Vec<_>>()
            .await;

        Ok(())
    }

    pub async fn scan_urls_with_tui(
        &self,
        urls: Vec<String>,
        tx: mpsc::Sender<TuiMessage>,
    ) -> Result<()> {
        let client = Arc::new(self.client.clone());

        stream::iter(urls)
            .map(|url| {
                let client = Arc::clone(&client);
                let tx = tx.clone();
                async move {
                    let _ = tx.send(TuiMessage::Scanned).await;

                    let start = Instant::now();
                    match client.request(&url, "GET", &[], None).await {
                        Ok(response) => {
                            let duration_ms = start.elapsed().as_millis() as u64;
                            let result = ScanResult::from_response(url.clone(), &response, duration_ms);

                            let tui_result = TuiResult {
                                url: result.url,
                                status_code: result.status_code,
                                content_length: result.content_length,
                                redirect_location: result.redirect_location,
                                content_type: result.content_type,
                                server: result.server,
                                duration_ms: result.duration_ms,
                            };

                            let _ = tx.send(TuiMessage::Result(tui_result)).await;
                        }
                        Err(_) => {
                            let _ = tx.send(TuiMessage::Error).await;
                        }
                    }
                }
            })
            .buffer_unordered(self.threads)
            .collect::<Vec<_>>()
            .await;

        let _ = tx.send(TuiMessage::Done).await;
        Ok(())
    }

    pub async fn detect_wildcard(&self, base_url: &str) -> Result<()> {
        let random_path = format!("{}/rustbuster-{}", base_url, uuid::Uuid::new_v4());
        
        match self.client.request(&random_path, "GET", &[], None).await {
            Ok(response) => {
                let status = response.status().as_u16();
                if status == 200 {
                    println!("[!] Warning: Wildcard response detected (Status: {})", status);
                    println!("[!] This may produce false positives");
                }
            }
            Err(_) => {}
        }

        Ok(())
    }

    pub fn get_discovered_dirs(&self) -> Vec<String> {
        self.discovered_dirs.clone()
    }
}
