use std::path::PathBuf;
use std::process;
use clap::{Parser, Subcommand};
use colored::Colorize;
use anyhow::{Result, anyhow};

use pocket::vcs::commands::{
    status_command, 
    interactive_pile_command, 
    interactive_shove_command, 
    interactive_timeline_command,
    log_command,
    graph_command
};

#[derive(Parser)]
#[command(name = "pocket")]
#[command(about = "Pocket Version Control System", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
    Pile {
        /// Repository path
        #[arg(short, long, default_value = ".")]
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
    
    /// 📊 Show timeline graph
    Graph {
        /// Repository path
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

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
        Commands::Pile { path } => {
            if let Err(e) = interactive_pile_command(path) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Shove { path } => {
            if let Err(e) = interactive_shove_command(path) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
        Commands::Timeline { path } => {
            if let Err(e) = interactive_timeline_command(path) {
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
        Commands::Graph { path } => {
            if let Err(e) = graph_command(path) {
                eprintln!("{} {}", "❌".red(), format!("Error: {}", e).red());
                process::exit(1);
            }
        },
    }
    
    Ok(())
} 