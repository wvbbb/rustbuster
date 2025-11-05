use crate::cli::DirArgs;
use crate::core::{Scanner, Wordlist};
use crate::output::tui;
use anyhow::Result;
use std::collections::HashSet;
use url::Url;

pub async fn run(args: DirArgs) -> Result<()> {
    let base_url = Url::parse(&args.url)?;
    
    if !args.common.no_tui {
        return run_with_tui(args, base_url).await;
    }
    
    if args.recursive {
        run_recursive(args, base_url).await
    } else {
        run_single(args, base_url).await
    }
}

async fn run_with_tui(args: DirArgs, base_url: Url) -> Result<()> {
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    
    let mut extensions = args.common.get_extensions(&args.extensions);
    if args.backup_extensions {
        extensions.extend(vec![
            ".bak".to_string(),
            ".backup".to_string(),
            ".old".to_string(),
            ".orig".to_string(),
            ".save".to_string(),
            ".swp".to_string(),
            ".tmp".to_string(),
            "~".to_string(),
        ]);
    }
    
    let words = if !extensions.is_empty() {
        wordlist.expand_with_extensions(&extensions)
    } else {
        wordlist.words.clone()
    };

    let urls: Vec<String> = words
        .iter()
        .map(|word| {
            let path = if word.starts_with('/') {
                word.clone()
            } else {
                format!("/{}", word)
            };
            
            let mut url = base_url.clone();
            url.set_path(&path);
            url.to_string()
        })
        .collect();

    let total = urls.len();
    let scanner = Scanner::new_from_common(args.common.clone())?;
    
    tui::run_tui_mode(
        "dir".to_string(),
        args.url.clone(),
        wordlist_path.clone(),
        args.common.threads,
        total,
        args.common.output.clone(),
        args.common.output_format.clone(),
        |tx| async move {
            scanner.scan_urls_with_tui(urls, tx).await
        },
    ).await
}

async fn run_single(args: DirArgs, base_url: Url) -> Result<()> {
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    
    let mut extensions = args.common.get_extensions(&args.extensions);
    
    if args.backup_extensions {
        extensions.extend(vec![
            ".bak".to_string(),
            ".backup".to_string(),
            ".old".to_string(),
            ".orig".to_string(),
            ".save".to_string(),
            ".swp".to_string(),
            ".tmp".to_string(),
            "~".to_string(),
        ]);
    }
    
    let words = if !extensions.is_empty() {
        wordlist.expand_with_extensions(&extensions)
    } else {
        wordlist.words.clone()
    };

    let urls: Vec<String> = words
        .iter()
        .map(|word| {
            let path = if word.starts_with('/') {
                word.clone()
            } else {
                format!("/{}", word)
            };
            
            let mut url = base_url.clone();
            url.set_path(&path);
            url.to_string()
        })
        .collect();

    let mut scanner = Scanner::new_from_common(args.common)?;
    scanner.detect_wildcard(base_url.as_str()).await?;
    scanner.scan_urls(urls).await?;

    Ok(())
}

async fn run_recursive(args: DirArgs, base_url: Url) -> Result<()> {
    let max_depth = args.depth;
    let mut scanned_dirs: HashSet<String> = HashSet::new();
    let mut dirs_to_scan: Vec<(String, usize)> = vec![(base_url.to_string(), 0)];
    
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    
    let mut extensions = args.common.get_extensions(&args.extensions);
    if args.backup_extensions {
        extensions.extend(vec![
            ".bak".to_string(),
            ".backup".to_string(),
            ".old".to_string(),
            ".orig".to_string(),
            ".save".to_string(),
            ".swp".to_string(),
            ".tmp".to_string(),
            "~".to_string(),
        ]);
    }
    
    let words = if !extensions.is_empty() {
        wordlist.expand_with_extensions(&extensions)
    } else {
        wordlist.words.clone()
    };

    while let Some((current_url, depth)) = dirs_to_scan.pop() {
        if depth > max_depth || scanned_dirs.contains(&current_url) {
            continue;
        }

        scanned_dirs.insert(current_url.clone());

        if !args.common.quiet {
            println!("\n[*] Scanning: {} (depth: {})", current_url, depth);
        }

        let current_base = Url::parse(&current_url)?;

        let urls: Vec<String> = words
            .iter()
            .map(|word| {
                let path = if word.starts_with('/') {
                    word.clone()
                } else {
                    format!("/{}", word)
                };
                
                let mut url = current_base.clone();
                let current_path = url.path().trim_end_matches('/');
                url.set_path(&format!("{}{}", current_path, path));
                url.to_string()
            })
            .collect();

        let mut scanner = Scanner::new_from_common(args.common.clone())?;
        
        if depth == 0 {
            scanner.detect_wildcard(current_base.as_str()).await?;
        }
        
        scanner.scan_urls(urls).await?;

        let discovered = scanner.get_discovered_dirs();
        for dir in discovered {
            if !scanned_dirs.contains(&dir) {
                dirs_to_scan.push((dir, depth + 1));
            }
        }
    }

    Ok(())
}
