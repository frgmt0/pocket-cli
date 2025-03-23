mod cards;
mod cli;
mod config;
mod errors;
mod logging;
mod models;
mod search;
mod storage;
mod utils;
mod version;

use cli::Cli;
use clap::Parser;
use errors::PocketResult;
use std::process;
use log::error;

fn main() {
    let cli = Cli::parse();
    
    if let Err(err) = run_app(cli) {
        error!("Error: {}", err);
        logging::error(&format!("{}", err));
        process::exit(1);
    }
}

fn run_app(cli: Cli) -> PocketResult<()> {
    cli::handler::handle_command(cli)
}
