use crate::cli::CommonArgs;
use anyhow::{Result, Context};
use reqwest::{Client, ClientBuilder, Response};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    user_agents: Option<Arc<Vec<String>>>,
    user_agent_index: Arc<AtomicUsize>,
}

impl HttpClient {
    pub fn new_from_common(args: &CommonArgs) -> Result<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(args.timeout))
            .user_agent(&args.user_agent)
            .danger_accept_invalid_certs(args.no_tls_validation);

        if !args.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        if let Some(proxy_url) = &args.proxy {
            let proxy = if proxy_url.starts_with("socks5://") || proxy_url.starts_with("socks4://") {
                reqwest::Proxy::all(proxy_url)
                    .context(format!("Failed to configure SOCKS proxy: {}", proxy_url))?
            } else if proxy_url.starts_with("http://") || proxy_url.starts_with("https://") {
                reqwest::Proxy::all(proxy_url)
                    .context(format!("Failed to configure HTTP proxy: {}", proxy_url))?
            } else {
                let full_url = format!("http://{}", proxy_url);
                reqwest::Proxy::all(&full_url)
                    .context(format!("Failed to configure proxy: {}", full_url))?
            };
            
            builder = builder.proxy(proxy);
            
            if args.verbose || !args.quiet {
                eprintln!("[+] Using proxy: {}", proxy_url);
            }
        }

        let client = builder.build()
            .context("Failed to build HTTP client")?;

        let user_agents = if let Some(ua_file) = &args.user_agents_file {
            let content = std::fs::read_to_string(ua_file)?;
            let agents: Vec<String> = content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
                .collect();
            
            if agents.is_empty() {
                None
            } else {
                if args.verbose {
                    eprintln!("[+] Loaded {} user agents for rotation", agents.len());
                }
                Some(Arc::new(agents))
            }
        } else {
            None
        };

        Ok(HttpClient {
            client,
            user_agents,
            user_agent_index: Arc::new(AtomicUsize::new(0)),
        })
    }

    fn get_user_agent(&self) -> Option<String> {
        self.user_agents.as_ref().map(|agents| {
            let index = self.user_agent_index.fetch_add(1, Ordering::SeqCst);
            agents[index % agents.len()].clone()
        })
    }

    pub async fn request(
        &self,
        url: &str,
        method: &str,
        headers: &[(String, String)],
        cookies: Option<&str>,
    ) -> Result<Response> {
        let mut request = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "HEAD" => self.client.head(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            "PATCH" => self.client.patch(url),
            _ => self.client.get(url),
        };

        if let Some(ua) = self.get_user_agent() {
            request = request.header("User-Agent", ua);
        }

        for (key, value) in headers {
            request = request.header(key, value);
        }

        if let Some(cookie_str) = cookies {
            request = request.header("Cookie", cookie_str);
        }

        let response = request.send().await?;
        Ok(response)
    }

    #[allow(dead_code)]
    pub async fn test_connection(&self, test_url: &str, verbose: bool) -> Result<bool> {
        if verbose {
            eprintln!("[*] Testing connection to: {}", test_url);
        }
        
        match self.client.get(test_url).send().await {
            Ok(response) => {
                if verbose {
                    eprintln!("[+] Connection test successful (Status: {})", response.status());
                }
                Ok(true)
            }
            Err(e) => {
                if verbose {
                    eprintln!("[!] Connection test failed: {}", e);
                    if e.is_timeout() {
                        eprintln!("[!] Error type: Timeout");
                    } else if e.is_connect() {
                        eprintln!("[!] Error type: Connection failed");
                    } else if e.is_request() {
                        eprintln!("[!] Error type: Request error");
                    }
                }
                Ok(false)
            }
        }
    }

    #[allow(dead_code)]
    pub async fn check_external_ip(&self) -> Result<String> {
        let ip_services = vec![
            "https://api.ipify.org",
            "https://ifconfig.me/ip",
            "https://icanhazip.com",
        ];

        for service in ip_services {
            if let Ok(response) = self.client.get(service).send().await {
                if let Ok(ip) = response.text().await {
                    let ip = ip.trim().to_string();
                    if !ip.is_empty() {
                        return Ok(ip);
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Failed to retrieve external IP address"))
    }
}

pub struct ScanResult {
    pub url: String,
    pub status_code: u16,
    pub content_length: u64,
    pub redirect_location: Option<String>,
    #[allow(dead_code)]
    pub body: Option<String>,
    pub content_type: Option<String>,
    pub server: Option<String>,
    pub duration_ms: u64,
}

impl ScanResult {
    pub fn from_response(url: String, response: &Response, duration_ms: u64) -> Self {
        let status_code = response.status().as_u16();
        let content_length = response.content_length().unwrap_or(0);
        let redirect_location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
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

        ScanResult {
            url,
            status_code,
            content_length,
            redirect_location,
            body: None,
            content_type,
            server,
            duration_ms,
        }
    }

    #[allow(dead_code)]
    pub async fn from_response_with_body(url: String, response: Response, duration_ms: u64) -> Self {
        let status_code = response.status().as_u16();
        let content_length = response.content_length().unwrap_or(0);
        let redirect_location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        
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

        let body = response.text().await.ok();

        ScanResult {
            url,
            status_code,
            content_length,
            redirect_location,
            body,
            content_type,
            server,
            duration_ms,
        }
    }
    
    pub fn status_text(&self) -> &'static str {
        match self.status_code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            301 => "Moved Permanently",
            302 => "Found",
            303 => "See Other",
            304 => "Not Modified",
            307 => "Temporary Redirect",
            308 => "Permanent Redirect",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            429 => "Too Many Requests",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            504 => "Gateway Timeout",
            _ => "Unknown",
        }
    }
}
