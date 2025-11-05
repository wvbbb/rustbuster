//! Wordlist management for directory and file enumeration.
//! 
//! This module handles loading wordlists from files and expanding them with extensions.
//! Wordlists are used as the basis for brute-forcing directories, files, subdomains, and vhosts.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represents a wordlist loaded from a file
pub struct Wordlist {
    pub words: Vec<String>,
}

impl Wordlist {
    /// Loads a wordlist from a file path
    /// 
    /// Filters out empty lines and comments (lines starting with #)
    pub fn from_file(path: &str) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open wordlist file: {}", path))?;
        
        let reader = BufReader::new(file);
        let words: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        if words.is_empty() {
            anyhow::bail!("Wordlist is empty or contains no valid entries");
        }

        Ok(Wordlist { words })
    }

    /// Returns the number of words in the wordlist
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Checks if the wordlist is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Expands the wordlist by appending file extensions to each word
    /// 
    /// For example, if the wordlist contains "admin" and extensions are [".php", ".html"],
    /// the result will be ["admin", "admin.php", "admin.html"]
    pub fn expand_with_extensions(&self, extensions: &[String]) -> Vec<String> {
        let mut expanded = Vec::new();
        
        for word in &self.words {
            expanded.push(word.clone());
            
            for ext in extensions {
                expanded.push(format!("{}{}", word, ext));
            }
        }
        
        expanded
    }
}
