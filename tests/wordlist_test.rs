//! Unit tests for wordlist functionality

use rustbuster::core::wordlist::Wordlist;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_wordlist_from_file() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "admin").unwrap();
    writeln!(file, "login").unwrap();
    writeln!(file, "test").unwrap();

    let wordlist = Wordlist::from_file(file.path().to_str().unwrap()).unwrap();
    assert_eq!(wordlist.len(), 3);
    assert!(wordlist.words.contains(&"admin".to_string()));
}
// ensure empty lines are ignored
#[test]
fn test_wordlist_filters_empty_lines() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "admin").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "login").unwrap();

    let wordlist = Wordlist::from_file(file.path().to_str().unwrap()).unwrap();
    assert_eq!(wordlist.len(), 2);
}

// ensure comment lines starting with '#' are ignored
#[test]
fn test_wordlist_filters_comments() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "admin").unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "login").unwrap();

    let wordlist = Wordlist::from_file(file.path().to_str().unwrap()).unwrap();
    assert_eq!(wordlist.len(), 2);
}

// test expansion with file extensions keeps originals and adds ext versions
#[test]
fn test_wordlist_expand_with_extensions() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "admin").unwrap();
    writeln!(file, "login").unwrap();

    let wordlist = Wordlist::from_file(file.path().to_str().unwrap()).unwrap();
    let extensions = vec![".php".to_string(), ".html".to_string()];
    let expanded = wordlist.expand_with_extensions(&extensions);

    assert_eq!(expanded.len(), 6);
    assert!(expanded.contains(&"admin".to_string()));
    assert!(expanded.contains(&"admin.php".to_string()));
    assert!(expanded.contains(&"admin.html".to_string()));
}

// empty wordlist file should return an error
#[test]
fn test_wordlist_empty_file() {
    let file = NamedTempFile::new().unwrap();
    let result = Wordlist::from_file(file.path().to_str().unwrap());
    assert!(result.is_err());
}
