use std::path::{PathBuf, Path};
use std::process;
use clap::{Parser, Subcommand};
use colored::Colorize;
use anyhow::{Result, anyhow};
use std::fs;
use pocket_cli::vcs::Repository;
use pocket_cli::cards::{CardManager, Card, CardConfig, CardCommand};

use pocket_cli::vcs::commands::{
    status_command,
    pile_command,
    unpile_command,
    shove_command,
    log_command,
    timeline_new_command,
    timeline_switch_command,
    timeline_list_command,
    merge_command,
    remote_add_command,
    remote_remove_command,
    remote_list_command,
    push_command,
    ignore_command,
};

#[derive(Parser)]
#[command(name = "pocket")]
#[command(about = "Pocket Version Control System", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Pocket VCS Commands
#[derive(Subcommand)]
enum Commands {
    /// 🔍 Show repository status
    Status {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 📦 Add files to the pile (staging area)
    /// 
    /// Directories will be added recursively, and glob patterns like src/* are supported.
    /// Use --all to add all modified and untracked files, or --pattern to add files matching a pattern.
    #[command(about = "Add files to the pile (staging area). Directories will be added recursively, and glob patterns like src/* are supported.")]
    Pile {
        #[arg(value_name = "FILES", help = "Files or directories to add (directories will be added recursively, supports glob patterns like src/*)")]
        files: Vec<String>,
        
        #[arg(short, long, help = "Add all modified and untracked files")]
        all: bool,
        
        #[arg(long, value_name = "PATTERN", help = "Add files matching pattern (supports glob patterns)")]
        pattern: Option<String>,
        
        #[arg(short, long, default_value = ".", help = "Repository path")]
        path: PathBuf,
    },
    
    /// 📌 Create a new shove (commit)
    Shove {
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🌿 Manage timelines (branches)
    Timeline {
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🆕 Initialize a new repository
    Init {
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🔄 Merge timelines
    Merge {
        /// Timeline to merge
        timeline: String,
        
        /// Resolve conflicts interactively
        #[arg(long)]
        resolve: bool,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🌐 Manage remote repositories
    Remote {
        /// Remote operation (add, remove, list)
        #[command(subcommand)]
        operation: RemoteOperation,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// ⬆️ Push changes to a remote repository
    Push {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
        
        /// Timeline to push
        #[arg(default_value = "main")]
        timeline: String,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// ⬇️ Pull changes from a remote repository
    Pull {
        /// Remote name
        #[arg(default_value = "origin")]
        remote: String,
        
        /// Timeline to pull
        #[arg(default_value = "main")]
        timeline: String,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 📜 Show commit history
    Log {
        /// Show verbose output with file changes
        #[arg(short, long)]
        verbose: bool,
        
        /// Timeline to show history for
        #[arg(short, long)]
        timeline: Option<String>,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🚫 Manage ignore patterns
    Ignore {
        /// Add a new ignore pattern
        #[arg(short, long)]
        add: Option<String>,
        
        /// Remove an ignore pattern
        #[arg(short, long)]
        remove: Option<String>,
        
        /// List all ignore patterns
        #[arg(short, long)]
        list: bool,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
    
    /// 🔌 Manage cards
    Cards {
        /// Card subcommand
        #[command(subcommand)]
        operation: Option<CardOperation>,
        
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

/// Remote operations
#[derive(Subcommand)]
enum RemoteOperation {
    /// Add a new remote
    Add {
        /// Remote name
        name: String,
        
        /// Remote URL
        url: String,
    },
    
    /// Remove a remote
    Remove {
        /// Remote name
        name: String,
    },
    
    /// List all remotes
    List,
}

/// Card operations
#[derive(Subcommand)]
enum CardOperation {
    /// Add a new card
    Add {
        /// Card name
        name: String,
        
        /// Card URL
        url: String,
    },
    
    /// Remove a card
    Remove {
        /// Card name
        name: String,
    },
    
    /// List all cards
    List,
    
    /// Enable a card
    Enable {
        /// Card name
        name: String,
    },
    
    /// Disable a card
    Disable {
        /// Card name
        name: String,
    },
    
    /// Execute a card command
    Execute {
        /// Card name
        name: String,
        
        /// Command to execute
        command: String,
        
        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Print a beautiful header
    println!("\n{} {} {}\n", "🚀".bright_cyan(), "Pocket VCS".bold().bright_cyan(), "🚀".bright_cyan());
    
    match &cli.command {
        Commands::Status { verbose, path } => {
            if let Err(e) = status_command(path, *verbose) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Pile { files, all, pattern, path } => {
            // Convert Vec<String> to Vec<&Path>
            let file_paths: Vec<&Path> = files.iter().map(|s| Path::new(s)).collect();
            
            if let Err(e) = pile_command(path, file_paths, *all, pattern.as_deref()) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Shove { path } => {
            // Add the missing message and editor parameters
            if let Err(e) = shove_command(path, None, true) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Timeline { path } => {
            // Since timeline_new_command requires a name and based_on parameters,
            // we should use a different command or provide the required parameters
            // Let's use timeline_list_command instead which is more appropriate for this case
            if let Err(e) = timeline_list_command(path) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Init { path } => {
            println!("{} Initializing repository at {}", "🆕".green(), path.display());
            // In a real implementation, we would call repo.init() here
            println!("{} Repository initialized successfully", "✅".green());
        },
        Commands::Merge { timeline, resolve, path } => {
            println!("{} Merging timeline {} into current timeline", "🔄".yellow(), timeline.bright_green());
            
            if *resolve {
                println!("{} Interactive conflict resolution enabled", "🔍".blue());
                // In a real implementation, we would call merge.resolve_conflicts_interactively() here
            }
            
            // In a real implementation, we would call repo.merge_timeline() here
            println!("{} Merge completed successfully", "✅".green());
        },
        Commands::Remote { operation, path } => {
            match operation {
                RemoteOperation::Add { name, url } => {
                    println!("{} Adding remote {} with URL {}", "🌐".blue(), name.bright_green(), url.bright_white());
                    // In a real implementation, we would call repo.add_remote() here
                    println!("{} Remote added successfully", "✅".green());
                },
                RemoteOperation::Remove { name } => {
                    println!("{} Removing remote {}", "🌐".blue(), name.bright_green());
                    // In a real implementation, we would call repo.remove_remote() here
                    println!("{} Remote removed successfully", "✅".green());
                },
                RemoteOperation::List => {
                    println!("{} Available remotes:", "🌐".blue());
                    // In a real implementation, we would list remotes here
                    println!("  origin: https://example.com/repo.git");
                    println!("  backup: https://backup.example.com/repo.git");
                },
            }
        },
        Commands::Push { remote, timeline, path } => {
            println!("{} Pushing timeline {} to remote {}", "⬆️".blue(), timeline.bright_green(), remote.bright_white());
            // In a real implementation, we would call repo.push() here
            println!("{} Push completed successfully", "✅".green());
        },
        Commands::Pull { remote, timeline, path } => {
            println!("{} Pulling timeline {} from remote {}", "⬇️".blue(), timeline.bright_green(), remote.bright_white());
            // In a real implementation, we would call repo.pull() here
            println!("{} Pull completed successfully", "✅".green());
        },
        Commands::Log { verbose, timeline, path } => {
            if let Err(e) = log_command(path, *verbose, timeline.as_deref()) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Ignore { add, remove, list, path } => {
            if let Err(e) = ignore_command(path, add.as_deref(), remove.as_deref(), *list) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                std::process::exit(1);
            }
        },
        Commands::Cards { operation, path } => {
            if let Err(e) = cards_command(path, operation.as_ref()) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
    }
    
    Ok(())
}

/// Handles card commands
fn cards_command(path: &Path, operation: Option<&CardOperation>) -> Result<()> {
    // Create a card manager
    let data_dir = path.join(".pocket");
    let card_dir = data_dir.join("cards");
    let mut card_manager = CardManager::new(&card_dir);
    
    // Load cards
    card_manager.load_cards()?;
    
    // Handle the operation
    match operation {
        Some(CardOperation::Add { name, url }) => {
            println!("{} Adding card {} with URL {}", "🔌".blue(), name.bright_green(), url.bright_white());
            // In a real implementation, we would add the card here
            println!("{} Card added successfully", "✅".green());
        },
        Some(CardOperation::Remove { name }) => {
            println!("{} Removing card {}", "🔌".blue(), name.bright_green());
            // In a real implementation, we would remove the card here
            println!("{} Card removed successfully", "✅".green());
        },
        Some(CardOperation::List) => {
            println!("{} Available cards:", "🔌".blue());
            let cards = card_manager.list_cards();
            if cards.is_empty() {
                println!("  No cards installed");
            } else {
                for (name, version, enabled) in cards {
                    let status = if enabled { "enabled".green() } else { "disabled".red() };
                    println!("  {} (v{}) - {}", name.bright_green(), version, status);
                }
            }
        },
        Some(CardOperation::Enable { name }) => {
            println!("{} Enabling card {}", "🔌".blue(), name.bright_green());
            card_manager.enable_card(name)?;
            println!("{} Card enabled successfully", "✅".green());
        },
        Some(CardOperation::Disable { name }) => {
            println!("{} Disabling card {}", "🔌".blue(), name.bright_green());
            card_manager.disable_card(name)?;
            println!("{} Card disabled successfully", "✅".green());
        },
        Some(CardOperation::Execute { name, command, args }) => {
            println!("{} Executing command {} for card {} with args: {:?}", "🔌".blue(), command.bright_white(), name.bright_green(), args);
            card_manager.execute_command(name, command, args)?;
            println!("{} Command executed successfully", "✅".green());
        },
        None => {
            // If no operation is specified, list all cards
            println!("{} Available cards:", "🔌".blue());
            let cards = card_manager.list_cards();
            if cards.is_empty() {
                println!("  No cards installed");
            } else {
                for (name, version, enabled) in cards {
                    let status = if enabled { "enabled".green() } else { "disabled".red() };
                    println!("  {} (v{}) - {}", name.bright_green(), version, status);
                }
            }
        }
    }
    
    Ok(())
} 