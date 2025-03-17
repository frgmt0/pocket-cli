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
use errors::{PocketError, PocketResult, IntoAnyhow};
use std::process;
use log::error;

fn main() {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Run the command
    if let Err(err) = run_app(cli) {
        // Log the error
        error!("Error: {}", err);
        
        // Display a user-friendly error message
        logging::error(&format!("{}", err));
        
        // Exit with an error code
        process::exit(1);
    }
}

/// Run the application with the given CLI arguments
fn run_app(cli: Cli) -> PocketResult<()> {
    // Handle the command
    cli::handler::handle_command(cli)
}
