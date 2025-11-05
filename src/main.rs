mod cli;
mod core;
mod modes;
mod output;
mod utils;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--show-args".to_string()) || args.contains(&"--arguments".to_string()) {
        cli::help::print_arguments_help();
        return Ok(());
    }
    
    if args.contains(&"--examples".to_string()) {
        cli::help::print_examples();
        return Ok(());
    }
    
    if args.contains(&"--info".to_string()) {
        cli::help::print_info();
        return Ok(());
    }
    
    utils::config::load_config();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Dir(args) => modes::dir::run(args).await?,
        Commands::Dns(args) => modes::dns::run(args).await?,
        Commands::Vhost(args) => modes::vhost::run(args).await?,
        Commands::Fuzz(args) => modes::fuzz::run(args).await?,
    }
    
    Ok(())
}
