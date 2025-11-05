use crate::cli::FuzzArgs;
use crate::core::{Scanner, Wordlist};
use crate::output::tui;
use anyhow::{Result, anyhow};

pub async fn run(args: FuzzArgs) -> Result<()> {
    if !args.url.contains("FUZZ") {
        return Err(anyhow!("URL must contain the FUZZ keyword (e.g., http://example.com/FUZZ)"));
    }
    
    let wordlist_path = args.common.wordlist.as_ref()
        .ok_or_else(|| anyhow!("Wordlist is required"))?;
    let wordlist = Wordlist::from_file(wordlist_path)?;
    
    let extensions = args.common.get_extensions(&args.extensions);
    let words = if !extensions.is_empty() {
        wordlist.expand_with_extensions(&extensions)
    } else {
        wordlist.words.clone()
    };

    let urls: Vec<String> = words
        .iter()
        .map(|word| args.url.replace("FUZZ", word))
        .collect();

    if !args.common.no_tui {
        let total = urls.len();
        let scanner = Scanner::new_from_common(args.common.clone())?;
        
        return tui::run_tui_mode(
            "fuzz".to_string(),
            args.url.clone(),
            wordlist_path.clone(),
            args.common.threads,
            total,
            args.common.output.clone(),
            args.common.output_format.clone(),
            |tx| async move {
                scanner.scan_urls_with_tui(urls, tx).await
            },
        ).await;
    }

    let mut scanner = Scanner::new_from_common(args.common)?;
    scanner.scan_urls(urls).await?;

    Ok(())
}
