mod cli;
mod constants;
mod engine;
mod event_format;
mod metrics;
mod model;
mod network;
mod quality;
mod stats;
mod storage;
#[cfg(feature = "tui")]
mod tui;
#[cfg(feature = "tui")]
mod update;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Cli::parse();
    let is_silent = args.silent;
    let is_non_tui = args.silent || args.json || args.text;

    match cli::run(args).await {
        Ok(()) => {
            if is_non_tui {
                std::process::exit(0);
            }
            Ok(())
        }
        Err(e) => {
            if is_silent {
                println!("{}", e);
                std::process::exit(1);
            } else {
                Err(e)
            }
        }
    }
}
