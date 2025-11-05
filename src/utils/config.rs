use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub default_threads: Option<usize>,
    pub default_timeout: Option<u64>,
    pub default_user_agent: Option<String>,
    pub default_wordlist: Option<String>,
    pub proxy: Option<String>,
}

impl Config {
    pub fn load() -> Option<Self> {
        let config_path = Self::get_config_path()?;
        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    }

    fn get_config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".rustbuster.toml"))
    }
}

pub fn load_config() {
    if let Some(config) = Config::load() {
        println!("[*] Loaded configuration from ~/.rustbuster.toml");
        if config.proxy.is_some() {
            println!("[*] Default proxy configured");
        }
    }
}
