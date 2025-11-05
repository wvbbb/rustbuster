use clap::{Parser, Subcommand};
use ansi_term::Style;

fn get_after_help() -> String {
    format!(
        "\n{}\n  rustbuster dir -u http://example.com -w wordlist.txt\n  rustbuster dns -d example.com -w subdomains.txt\n  rustbuster vhost -u http://example.com -w vhosts.txt\n  rustbuster fuzz -u http://example.com/FUZZ -w wordlist.txt\n\n{}\n  --arguments    Show all available arguments and options\n  --examples     Show detailed usage examples for all modes\n  --info         Show additional information about Rustbuster\n\nFor mode-specific help: rustbuster <MODE> --help\n",
        Style::new().bold().underline().paint("QUICK START:"),
        Style::new().bold().underline().paint("EXTRA INFO:")
    )
}

#[derive(Parser, Debug)]
#[command(name = "rustbuster")]
#[command(author = "wvbb")]
#[command(version = "0.1.0")]
#[command(about = "🦀 Rustbuster - A fast, professional web enumeration tool written in Rust", long_about = None)]
#[command(subcommand_help_heading = "MODES")]
#[command(after_help = get_after_help())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Dir(DirArgs),
    Dns(DnsArgs),
    Vhost(VhostArgs),
    Fuzz(FuzzArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct CommonArgs {
    #[arg(short = 'w', long, value_name = "FILE")]
    pub wordlist: Option<String>,

    #[arg(short = 't', long, default_value = "10", value_name = "NUM")]
    pub threads: usize,

    #[arg(long, default_value = "10", value_name = "SECS")]
    pub timeout: u64,

    #[arg(long)]
    pub no_tui: bool,

    #[arg(short = 's', long, default_value = "200,204,301,302,307,401,403", value_name = "CODES")]
    pub status_codes: String,

    #[arg(short = 'n', long, value_name = "CODES")]
    pub negative_status_codes: Option<String>,

    #[arg(short = 'r', long)]
    pub follow_redirects: bool,

    #[arg(short = 'a', long, default_value = "rustbuster/0.1.0", value_name = "STRING")]
    pub user_agent: String,

    #[arg(long, value_name = "FILE")]
    pub user_agents_file: Option<String>,

    #[arg(long, default_value = "GET", value_name = "METHOD")]
    pub method: String,

    #[arg(short = 'c', long, value_name = "STRING")]
    pub cookies: Option<String>,

    #[arg(short = 'H', long, value_name = "HEADER")]
    pub headers: Vec<String>,

    #[arg(short = 'p', long, value_name = "URL")]
    pub proxy: Option<String>,

    #[arg(long)]
    pub no_tls_validation: bool,

    #[arg(short = 'e', long)]
    pub expanded: bool,

    #[arg(short = 'q', long)]
    pub quiet: bool,

    #[arg(short = 'v', long)]
    pub verbose: bool,

    #[arg(long)]
    pub no_progress: bool,

    #[arg(short = 'o', long, value_name = "FILE")]
    pub output: Option<String>,

    #[arg(long, default_value = "plain", value_name = "FORMAT")]
    pub output_format: String,

    #[arg(long)]
    pub wildcard: bool,

    #[arg(long, value_name = "REGEX")]
    pub filter_regex: Option<String>,

    #[arg(long, value_name = "REGEX")]
    pub match_regex: Option<String>,

    #[arg(long, value_name = "SIZES")]
    pub filter_size: Option<String>,

    #[arg(long, value_name = "MS")]
    pub delay: Option<u64>,
    
    #[arg(long, value_name = "NAME")]
    pub save_session: Option<String>,
    
    #[arg(long, value_name = "NAME")]
    pub resume_session: Option<String>,
    
    #[arg(long)]
    pub smart_404: bool,
    
    #[arg(long, value_name = "FILE")]
    pub targets: Option<String>,
    
    #[arg(long, value_name = "FILE")]
    pub report: Option<String>,
    
    #[arg(long, value_name = "FLOAT")]
    pub similarity_threshold: Option<f32>,
}

#[derive(Parser, Debug, Clone)]
pub struct DirArgs {
    #[arg(short = 'u', long, value_name = "URL")]
    pub url: String,

    #[arg(short = 'x', long, value_name = "EXTS")]
    pub extensions: Option<String>,

    #[arg(short = 'R', long)]
    pub recursive: bool,

    #[arg(long, default_value = "3", value_name = "NUM")]
    pub depth: usize,

    #[arg(long)]
    pub backup_extensions: bool,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct DnsArgs {
    #[arg(short = 'd', long, value_name = "DOMAIN")]
    pub domain: String,

    #[arg(long)]
    pub show_cname: bool,

    #[arg(long)]
    pub show_ips: bool,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct VhostArgs {
    #[arg(short = 'u', long, value_name = "URL")]
    pub url: String,

    #[command(flatten)]
    pub common: CommonArgs,
}

#[derive(Parser, Debug, Clone)]
pub struct FuzzArgs {
    #[arg(short = 'u', long, value_name = "URL")]
    pub url: String,

    #[arg(short = 'x', long, value_name = "EXTS")]
    pub extensions: Option<String>,

    #[command(flatten)]
    pub common: CommonArgs,
}

impl CommonArgs {
    pub fn get_status_codes(&self) -> Vec<u16> {
        self.status_codes
            .split(',')
            .filter_map(|s| s.trim().parse::<u16>().ok())
            .collect()
    }

    pub fn get_negative_status_codes(&self) -> Vec<u16> {
        self.negative_status_codes
            .as_ref()
            .map(|codes| {
                codes
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u16>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_extensions(&self, extensions_arg: &Option<String>) -> Vec<String> {
        extensions_arg
            .as_ref()
            .map(|exts| {
                exts.split(',')
                    .map(|s| {
                        let trimmed = s.trim();
                        if trimmed.starts_with('.') {
                            trimmed.to_string()
                        } else {
                            format!(".{}", trimmed)
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
