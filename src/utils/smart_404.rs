use crate::core::http_client::HttpClient;
use anyhow::Result;
use sha2::{Sha256, Digest};
use std::collections::HashSet;

/// Detects false positive responses by comparing against baseline patterns
#[allow(dead_code)]
pub struct Smart404Detector {
    baseline_hashes: HashSet<String>,
    baseline_sizes: HashSet<u64>,
    enabled: bool,
}

#[allow(dead_code)]
impl Smart404Detector {
    /// Creates a new detector instance
    pub fn new(enabled: bool) -> Self {
        Smart404Detector {
            baseline_hashes: HashSet::new(),
            baseline_sizes: HashSet::new(),
            enabled,
        }
    }

    /// Calibrates the detector by testing random non-existent paths
    /// 
    /// This establishes baseline patterns for 404 responses that may return 200 OK
    pub async fn calibrate(&mut self, client: &HttpClient, base_url: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        println!("[*] Calibrating smart 404 detection...");

        let test_paths = vec![
            format!("{}/rustbuster-404-test-{}", base_url.trim_end_matches('/'), uuid::Uuid::new_v4()),
            format!("{}/nonexistent-{}.html", base_url.trim_end_matches('/'), uuid::Uuid::new_v4()),
            format!("{}/missing-{}.php", base_url.trim_end_matches('/'), uuid::Uuid::new_v4()),
        ];

        for path in test_paths {
            if let Ok(response) = client.request(&path, "GET", &[], None).await {
                if let Ok(body) = response.text().await {
                    let hash = self.hash_content(&body);
                    self.baseline_hashes.insert(hash);
                    self.baseline_sizes.insert(body.len() as u64);
                }
            }
        }

        if !self.baseline_hashes.is_empty() {
            println!("[+] Smart 404 detection calibrated with {} baseline patterns", self.baseline_hashes.len());
        }

        Ok(())
    }

    /// Checks if a response matches the baseline 404 patterns
    pub fn is_false_positive(&self, body: &str, size: u64) -> bool {
        if !self.enabled {
            return false;
        }

        let hash = self.hash_content(body);
        self.baseline_hashes.contains(&hash) || self.baseline_sizes.contains(&size)
    }

    /// Hashes response content for comparison
    fn hash_content(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
