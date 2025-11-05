use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Represents a scan session that can be saved and resumed
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub target: String,
    pub wordlist: String,
    pub completed_words: Vec<String>,
    pub total_words: usize,
    pub found_results: Vec<SessionResult>,
}

/// A result found during a scan session
#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionResult {
    pub url: String,
    pub status_code: u16,
    pub content_length: u64,
}

#[allow(dead_code)]
impl Session {
    /// Creates a new scan session
    pub fn new(name: String, target: String, wordlist: String, total_words: usize) -> Self {
        let now = Utc::now();
        Session {
            name,
            created_at: now,
            last_updated: now,
            target,
            wordlist,
            completed_words: Vec::new(),
            total_words,
            found_results: Vec::new(),
        }
    }

    /// Saves the session to disk
    pub fn save(&mut self) -> Result<()> {
        self.last_updated = Utc::now();
        let session_dir = Self::get_session_dir()?;
        fs::create_dir_all(&session_dir)?;
        
        let session_file = session_dir.join(format!("{}.json", self.name));
        let json = serde_json::to_string_pretty(self)?;
        fs::write(session_file, json)?;
        
        Ok(())
    }

    /// Loads a session from disk by name
    pub fn load(name: &str) -> Result<Self> {
        let session_dir = Self::get_session_dir()?;
        let session_file = session_dir.join(format!("{}.json", name));
        
        let json = fs::read_to_string(&session_file)
            .context(format!("Failed to load session: {}", name))?;
        let session: Session = serde_json::from_str(&json)?;
        
        Ok(session)
    }

    /// Marks a word as completed in the session
    pub fn add_completed_word(&mut self, word: String) {
        self.completed_words.push(word);
    }

    /// Adds a found result to the session
    pub fn add_result(&mut self, result: SessionResult) {
        self.found_results.push(result);
    }

    /// Checks if a word has already been scanned
    pub fn is_word_completed(&self, word: &str) -> bool {
        self.completed_words.contains(&word.to_string())
    }

    /// Calculates scan progress as a percentage
    pub fn get_progress(&self) -> f32 {
        if self.total_words == 0 {
            return 0.0;
        }
        (self.completed_words.len() as f32 / self.total_words as f32) * 100.0
    }

    /// Gets the directory where sessions are stored
    fn get_session_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        Ok(home.join(".rustbuster").join("sessions"))
    }

    /// Lists all saved sessions
    pub fn list_sessions() -> Result<Vec<String>> {
        let session_dir = Self::get_session_dir()?;
        if !session_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        for entry in fs::read_dir(session_dir)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    sessions.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
        Ok(sessions)
    }
}
