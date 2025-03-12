mod commands;
mod models;
mod search;
mod storage;
mod utils;
mod version;
mod vcs;
mod cards;

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use colored::Colorize;
use std::path::Path;
use pocket_cli::cards::{CardManager, Card, CardConfig, CardCommand};

#[derive(Parser)]
#[command(
    name = "pocket",
    about = "A CLI tool for saving, organizing, and retrieving code snippets with integrated version control",
    version = version::VERSION_STRING,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Snippet Management Commands")]
    #[command(visible_alias = "snippet")]
    /// Add content to your pocket storage
    Add {
        /// Path to the file to add
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Specify text directly
        #[arg(short, long, value_name = "TEXT")]
        message: Option<String>,

        /// Open the default editor
        #[arg(short, long)]
        editor: bool,

        /// Add directly to a specific backpack
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,
    },

    /// Display all pocket entries
    List {
        /// Include entries from all backpacks
        #[arg(long)]
        include_backpacks: bool,

        /// Show entries from a specific backpack
        #[arg(long, value_name = "NAME")]
        backpack: Option<String>,

        /// Display in JSON format for scripting
        #[arg(long)]
        json: bool,
    },

    /// Remove an entry from storage
    Remove {
        /// ID of the entry to remove
        #[arg(value_name = "ID")]
        id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,

        /// Specify which backpack to remove from
        #[arg(long, value_name = "NAME")]
        backpack: Option<String>,
    },

    /// Create a new backpack for organizing entries
    Create {
        #[command(subcommand)]
        entity: CreateCommands,
    },

    /// Find entries using semantic similarity
    Search {
        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Limit the number of results
        #[arg(long, value_name = "N", default_value = "5")]
        limit: usize,

        /// Search only within a specific backpack
        #[arg(long, value_name = "NAME")]
        backpack: Option<String>,

        /// Use exact text matching instead of semantic search
        #[arg(long)]
        exact: bool,
    },

    /// Insert an entry into a file
    Insert {
        /// ID of the entry to insert
        #[arg(value_name = "ID")]
        id: Option<String>,

        /// Path to the file to insert into
        #[arg(value_name = "FILE")]
        file: Option<String>,

        /// Use the top search result
        #[arg(long)]
        top: bool,

        /// Skip confirmation
        #[arg(long)]
        no_confirm: bool,

        /// Custom delimiter for inserted content
        #[arg(long, value_name = "TEXT")]
        delimiter: Option<String>,
    },

    /// Reload all extensions
    Reload,

    /// Display help information
    ShowHelp {
        /// Show help for a specific command
        #[arg(value_name = "COMMAND")]
        command: Option<String>,

        /// List all installed extensions with descriptions
        #[arg(long)]
        extensions: bool,
    },

    /// Create and execute command chains
    Lint {
        /// The workflow string to execute
        #[arg(value_name = "WORKFLOW")]
        workflow: Option<String>,
    },

    /// Remove a saved workflow
    DeleteWorkflow {
        /// Name of the workflow to delete
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Display version information
    Version,

    /// Edit an existing entry
    Edit {
        /// ID of the entry to edit
        #[arg(value_name = "ID")]
        id: String,

        /// Specify which backpack to edit from
        #[arg(long, value_name = "NAME")]
        backpack: Option<String>,
    },

    /// Execute a script
    Execute {
        /// ID of the script to execute
        #[arg(value_name = "ID")]
        id: Option<String>,
        
        /// Path to the script file to execute
        #[arg(short, long, value_name = "FILE")]
        file: Option<String>,
        
        /// Execute a script from a specific backpack
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,
        
        /// Skip confirmation before executing script
        #[arg(long)]
        no_confirm: bool,
        
        /// Arguments to pass to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    #[command(about = "Version Control Commands")]
    #[command(visible_alias = "vcs")]
    /// Create a new repository
    NewRepo {
        /// Path where to create the repository
        #[arg(value_name = "PATH")]
        path: Option<String>,

        /// Initialize with a template
        #[arg(long, value_name = "TEMPLATE")]
        template: Option<String>,

        /// Don't create default files
        #[arg(long)]
        no_default: bool,
    },

    /// Show repository status
    Status {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Add files to the pile (staging area)
    Pile {
        /// Files to add
        #[arg(value_name = "FILES")]
        files: Vec<String>,

        /// Add all changes
        #[arg(short, long)]
        all: bool,

        /// Add files matching pattern
        #[arg(long, value_name = "PATTERN")]
        pattern: Option<String>,
    },

    /// Remove files from the pile (staging area)
    Unpile {
        /// Files to remove
        #[arg(value_name = "FILES")]
        files: Vec<String>,

        /// Remove all files
        #[arg(short, long)]
        all: bool,
    },

    /// Create a shove (commit)
    Shove {
        /// Commit message
        #[arg(short, long, value_name = "MESSAGE")]
        message: Option<String>,

        /// Open editor for message
        #[arg(short, long)]
        editor: bool,
    },

    /// Show commit history
    Log {
        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,
        
        /// Timeline to show history for
        #[arg(short, long)]
        timeline: Option<String>,
    },

    /// Manage timelines (branches)
    Timeline {
        #[command(subcommand)]
        action: TimelineCommands,
    },

    /// Merge a timeline into the current one
    Merge {
        /// Name of the timeline to merge
        #[arg(value_name = "NAME")]
        name: String,

        /// Merge strategy
        #[arg(long, value_name = "STRATEGY")]
        strategy: Option<String>,
    },

    /// Manage remote repositories
    Remote {
        #[command(subcommand)]
        action: RemoteCommands,
    },

    /// Fetch from a remote repository
    Fish {
        /// Name of the remote
        #[arg(value_name = "REMOTE")]
        remote: Option<String>,
    },

    /// Push to a remote repository
    Push {
        /// Name of the remote
        #[arg(value_name = "REMOTE")]
        remote: Option<String>,

        /// Name of the timeline to push
        #[arg(value_name = "TIMELINE")]
        timeline: Option<String>,
    },

    /// Manage ignore patterns
    Ignore {
        /// Pattern to add to ignore list
        #[arg(short, long, value_name = "PATTERN")]
        pattern: Option<String>,

        /// Pattern to remove from ignore list
        #[arg(short, long, value_name = "PATTERN")]
        remove: Option<String>,

        /// List all ignore patterns
        #[arg(short, long)]
        list: bool,
    },

    /// 🔌 Manage cards
    Cards {
        /// Card subcommand
        #[command(subcommand)]
        operation: Option<CardOperation>,
    }
}

#[derive(Subcommand)]
enum CreateCommands {
    /// Create a new backpack for organizing entries
    Backpack {
        /// Name of the backpack to create
        #[arg(value_name = "NAME")]
        name: String,

        /// Add a description for the backpack
        #[arg(long, value_name = "TEXT")]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
enum TimelineCommands {
    /// Create a new timeline
    New {
        /// Name of the timeline
        #[arg(value_name = "NAME")]
        name: String,

        /// Base the timeline on a specific shove
        #[arg(long, value_name = "SHOVE_ID")]
        based_on: Option<String>,
    },

    /// Switch to a timeline
    Switch {
        /// Name of the timeline
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List all timelines
    List,
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Add a remote repository
    Add {
        /// Name of the remote
        #[arg(value_name = "NAME")]
        name: String,

        /// URL of the remote
        #[arg(value_name = "URL")]
        url: String,
    },

    /// Remove a remote repository
    Remove {
        /// Name of the remote
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// List remote repositories
    List,
}

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
    
    /// Run a card command
    Run {
        /// Card name
        name: String,
        
        /// Command to execute
        command: String,
        
        /// Arguments for the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Create a new local card
    Create {
        /// Card name
        name: String,
        
        /// Card description
        description: String,
    },
    
    /// Build a card
    Build {
        /// Card name
        name: String,
        
        /// Build in release mode
        #[arg(long, short)]
        release: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle custom help display
    if let Commands::ShowHelp { command, extensions } = &cli.command {
        if command.is_none() && !extensions {
            print_custom_help();
            return Ok(());
        }
    }

    match cli.command {
        Commands::Add { file, message, editor, backpack } => {
            let id = commands::add_command(file, message, editor, backpack)?;
            println!("Entry added successfully with ID: {}", id);
        }
        Commands::List { include_backpacks, backpack, json } => {
            commands::list_command(include_backpacks, backpack, json)?;
        }
        Commands::Remove { id, force, backpack } => {
            commands::remove_command(id, force, backpack)?;
        }
        Commands::Create { entity } => {
            match entity {
                CreateCommands::Backpack { name, description } => {
                    commands::create_backpack_command(name, description)?;
                }
            }
        }
        Commands::Search { query, limit, backpack, exact } => {
            commands::search_command(query, limit, backpack, exact)?;
        }
        Commands::Insert { id, file, top, no_confirm, delimiter } => {
            commands::insert_command(id, file, top, no_confirm, delimiter)?;
        }
        Commands::Reload => {
            println!("Extension reloading is not yet implemented");
        }
        Commands::ShowHelp { command, extensions } => {
            commands::help_command(command, extensions)?;
        }
        Commands::Lint { workflow } => {
            commands::lint_command(workflow)?;
        }
        Commands::DeleteWorkflow { name } => {
            commands::delete_workflow_command(name)?;
        }
        Commands::Version => {
            let version = version::get_version();
            println!("{}", version);
            println!("\nProject: Pocket");
            println!("Version: {}", version.letter);
            
            if let Some(compat) = version.compatibility {
                println!("Compatibility: {}", compat);
            } else {
                println!("Compatibility: Fully compatible with all previous versions");
            }
            
            println!("Stability: {} ({})", version.stability, match version.stability {
                version::Stability::Alpha => "Experimental and seeking feedback",
                version::Stability::Beta => "Still buggy but not completely unusable",
                version::Stability::Candidate => "Almost ready for official release",
                version::Stability::Release => "Stable and ready for production use",
            });
            
            println!("Release Name: {}", version.name);
            println!("Internal Date: {}", version.date);
            println!("Cargo SemVer: {} (required for Rust ecosystem)", version.semver);
            println!("\nFor full changelog, see: https://github.com/frgmt0/pocket/blob/main/CHANGELOG.md");
        }
        Commands::Edit { id, backpack } => {
            commands::edit_command(id, backpack)?;
        }
        Commands::Execute { id, file, backpack, no_confirm, args } => {
            commands::execute_command(id, file, backpack, no_confirm, args)?;
        }
        // VCS commands
        Commands::NewRepo { path, template, no_default } => {
            let path_str = path.unwrap_or_else(|| ".".to_string());
            let path = std::path::Path::new(&path_str);
            vcs::commands::new_repo_command(path, template.as_deref(), no_default)?;
        }
        Commands::Status { verbose } => {
            let path = std::path::Path::new(".");
            vcs::commands::status_command(path, verbose)?;
        }
        Commands::Pile { files, all, pattern } => {
            let path = std::path::Path::new(".");
            let file_paths: Vec<&std::path::Path> = files.iter().map(|f| std::path::Path::new(f)).collect();
            vcs::commands::pile_command(path, file_paths, all, pattern.as_deref())?;
        }
        Commands::Unpile { files, all } => {
            let path = std::path::Path::new(".");
            let file_paths: Vec<&std::path::Path> = files.iter().map(|f| std::path::Path::new(f)).collect();
            vcs::commands::unpile_command(path, file_paths, all)?;
        }
        Commands::Shove { message, editor } => {
            let path = std::path::Path::new(".");
            vcs::commands::shove_command(path, message.as_deref(), editor)?;
        }
        Commands::Log { verbose, timeline } => {
            let path = std::path::Path::new(".");
            vcs::commands::log_command(path, verbose, timeline.as_deref())?;
        }
        Commands::Timeline { action } => {
            let path = std::path::Path::new(".");
            match action {
                TimelineCommands::New { name, based_on } => {
                    vcs::commands::timeline_new_command(path, &name, based_on.as_deref())?;
                }
                TimelineCommands::Switch { name } => {
                    vcs::commands::timeline_switch_command(path, &name)?;
                }
                TimelineCommands::List => {
                    vcs::commands::timeline_list_command(path)?;
                }
            }
        }
        Commands::Merge { name, strategy } => {
            let path = std::path::Path::new(".");
            vcs::commands::merge_command(path, &name, strategy.as_deref())?;
        }
        Commands::Remote { action } => {
            let path = std::path::Path::new(".");
            match action {
                RemoteCommands::Add { name, url } => {
                    vcs::commands::remote_add_command(path, &name, &url)?;
                }
                RemoteCommands::Remove { name } => {
                    vcs::commands::remote_remove_command(path, &name)?;
                }
                RemoteCommands::List => {
                    vcs::commands::remote_list_command(path)?;
                }
            }
        }
        Commands::Fish { remote } => {
            let path = std::path::Path::new(".");
            vcs::commands::fish_command(path, remote.as_deref())?;
        }
        Commands::Push { remote, timeline } => {
            let path = std::path::Path::new(".");
            vcs::commands::push_command(path, remote.as_deref(), timeline.as_deref())?;
        }
        Commands::Ignore { pattern, remove, list } => {
            let path = std::path::Path::new(".");
            vcs::commands::ignore_command(path, pattern.as_deref(), remove.as_deref(), list)?;
        }
        Commands::Cards { operation } => {
            cards_command(operation.as_ref())?;
        }
    }

    Ok(())
}

fn print_custom_help() {
    println!("pocket {}", version::VERSION_STRING);
    println!("A CLI tool for saving, organizing, and retrieving code snippets with integrated version control\n");
    
    println!("USAGE:");
    println!("    pocket <COMMAND>\n");
    
    println!("SNIPPET MANAGEMENT COMMANDS:");
    println!("    add                 Add content to your pocket storage");
    println!("    list                Display all pocket entries");
    println!("    remove              Remove an entry from storage");
    println!("    create              Create a new backpack for organizing entries");
    println!("    search              Find entries using semantic similarity");
    println!("    insert              Insert an entry into a file");
    println!("    edit                Edit an existing entry");
    println!("    execute             Execute a script");
    println!("    lint                Create and execute command chains");
    println!("    delete-workflow     Remove a saved workflow");
    println!("");
    
    println!("VERSION CONTROL COMMANDS:");
    println!("    new-repo            Create a new repository");
    println!("    status              Show repository status");
    println!("    pile                Add files to the pile (staging area)");
    println!("    unpile              Remove files from the pile (staging area)");
    println!("    shove               Create a shove (commit)");
    println!("    log                 Show commit history");
    println!("    timeline            Manage timelines (branches)");
    println!("    merge               Merge a timeline into the current one");
    println!("    remote              Manage remote repositories");
    println!("    fish                Fetch from a remote repository");
    println!("    push                Push to a remote repository");
    println!("    ignore              Manage ignore patterns");
    println!("");
    
    println!("GENERAL COMMANDS:");
    println!("    reload              Reload all extensions");
    println!("    help                Display help information");
    println!("    version             Display version information");
    println!("    cards               Manage cards (cards)");
    println!("");
    
    println!("For more information about a specific command, run:");
    println!("    pocket help <COMMAND>");
}

/// Handles card commands
fn cards_command(operation: Option<&CardOperation>) -> Result<()> {
    // Create a card manager
    let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let data_dir = home_dir.join(".pocket");
    let card_dir = data_dir.join("cards");
    let wallet_dir = data_dir.join("wallet");
    let mut card_manager = CardManager::new(&card_dir);
    
    // Ensure the wallet directory exists
    if !wallet_dir.exists() {
        std::fs::create_dir_all(&wallet_dir)
            .context("Failed to create wallet directory")?;
    }
    
    // Load cards
    card_manager.load_cards()?;
    
    // Handle the operation
    match operation {
        Some(CardOperation::Add { name, url }) => {
            println!("{} Adding card {} from {}", "🔌".blue(), name.bright_green(), url.bright_white());
            
            // Check if the card already exists
            if card_manager.card_exists(name) {
                println!("{} Card with name '{}' already exists", "❌".red(), name);
                return Ok(());
            }
            
            // Validate URL format (simple check for GitHub URL)
            if !url.starts_with("https://github.com/") && !url.starts_with("http://github.com/") {
                println!("{} Currently only GitHub URLs are supported (format: https://github.com/username/repo)", "❌".red());
                return Ok(());
            }
            
            // Create a temporary directory for cloning
            let temp_dir = std::env::temp_dir().join(format!("pocket-card-{}", name));
            if temp_dir.exists() {
                std::fs::remove_dir_all(&temp_dir)
                    .context("Failed to clean up temporary directory")?;
            }
            std::fs::create_dir_all(&temp_dir)
                .context("Failed to create temporary directory")?;
            
            // Clone the repository
            println!("{} Cloning repository...", "⏳".yellow());
            let status = std::process::Command::new("git")
                .args(&["clone", url, "--depth=1", "."])
                .current_dir(&temp_dir)
                .status()
                .context("Failed to execute git clone command")?;
            
            if !status.success() {
                println!("{} Failed to clone repository", "❌".red());
                return Ok(());
            }
            
            // Check for card.toml file
            let card_toml_path = temp_dir.join("card.toml");
            if !card_toml_path.exists() {
                println!("{} Repository does not contain a card.toml file", "❌".red());
                return Ok(());
            }
            
            // Check for src/lib.rs or main card file
            let lib_rs_path = temp_dir.join("src").join("lib.rs");
            if !lib_rs_path.exists() {
                println!("{} Repository does not contain a src/lib.rs file", "❌".red());
                return Ok(());
            }
            
            // Copy the card to the extensions directory
            let card_dir = wallet_dir.join(name);
            if card_dir.exists() {
                std::fs::remove_dir_all(&card_dir)
                    .context("Failed to remove existing card directory")?;
            }
            
            // Use a recursive copy function to copy the repository
            copy_dir_all(&temp_dir, &card_dir)
                .context("Failed to copy card files")?;
            
            // Clean up temporary directory
            std::fs::remove_dir_all(&temp_dir)
                .context("Failed to clean up temporary directory")?;
            
            // Register the card in the configuration
            card_manager.register_card_config(name, url)?;
            
            println!("{} Card added successfully", "✅".green());
            println!("To use the card, you need to build it with: pocket cards build {}", name);
        },
        Some(CardOperation::Remove { name }) => {
            println!("{} Removing card {}", "🔌".blue(), name.bright_green());
            
            // Check if the card exists
            if !card_manager.card_exists(name) {
                println!("{} Card '{}' not found", "❌".red(), name);
                return Ok(());
            }
            
            // Remove the card directory
            let card_dir = wallet_dir.join(name);
            if card_dir.exists() {
                std::fs::remove_dir_all(&card_dir)
                    .context("Failed to remove card directory")?;
            }
            
            // Remove the card from the configuration
            card_manager.remove_card_config(name)?;
            
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
        Some(CardOperation::Run { name, command, args }) => {
            println!("{} Running command {} for card {} with args: {:?}", "🔌".blue(), command.bright_white(), name.bright_green(), args);
            card_manager.execute_command(name, command, args)?;
            println!("{} Command executed successfully", "✅".green());
        },
        Some(CardOperation::Create { name, description }) => {
            println!("{} Creating local card {}", "🔌".blue(), name.bright_green());
            
            // Check if the name contains only valid characters
            if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                println!("{} Card name must contain only alphanumeric characters, underscores, or hyphens", "❌".red());
                return Ok(());
            }
            
            // Check if a card with this name already exists
            let card_dir = wallet_dir.join(name);
            if card_dir.exists() {
                println!("{} A card with the name '{}' already exists", "❌".red(), name);
                return Ok(());
            }
            
            // Check for cards with similar names to avoid confusion
            let entries = std::fs::read_dir(&wallet_dir)
                .context("Failed to read wallet directory")?;
            
            let similar_cards: Vec<String> = entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    if let Some(file_name) = entry.file_name().to_str() {
                        // Check if names are similar (e.g., "format" and "formatter")
                        file_name.contains(name) || name.contains(file_name)
                    } else {
                        false
                    }
                })
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();
            
            if !similar_cards.is_empty() {
                println!("{} Warning: Found cards with similar names:", "⚠️".yellow());
                for card in &similar_cards {
                    println!("  - {}", card);
                }
                println!("This might cause confusion. Do you want to continue? (y/N)");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)
                    .context("Failed to read user input")?;
                
                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("{} Card creation cancelled", "❌".red());
                    return Ok(());
                }
            }
            
            // Create the card directory structure
            std::fs::create_dir_all(&card_dir)
                .context("Failed to create card directory")?;
            std::fs::create_dir_all(card_dir.join("src"))
                .context("Failed to create card src directory")?;
            
            // Create card.toml
            let card_toml = format!(
                r#"[card]
name = "{}"
version = "0.1.0"
description = "{}"
author = ""

[dependencies]
# Add your dependencies here
# Example: serde = {{ version = "1.0", features = ["derive"] }}
"#,
                name,
                description
            );
            
            std::fs::write(card_dir.join("card.toml"), card_toml)
                .context("Failed to write card.toml")?;
            
            // Create Cargo.toml
            let cargo_toml = format!(
                r#"[package]
name = "pocket-card-{}"
version = "0.1.0"
edition = "2021"
description = "{}"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
# Add your dependencies here
"#,
                name,
                description
            );
            
            std::fs::write(card_dir.join("Cargo.toml"), cargo_toml)
                .context("Failed to write Cargo.toml")?;
            
            // Create lib.rs with template code
            let lib_rs = r#"use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardConfig {
    pub name: String,
    pub enabled: bool,
    pub options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct CardCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub struct Card {
    name: String,
    version: String,
    description: String,
    config: CardConfig,
}

impl Card {
    pub fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").replace("pocket-card-", ""),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            config: CardConfig {
                name: env!("CARGO_PKG_NAME").replace("pocket-card-", ""),
                enabled: true,
                options: HashMap::new(),
            },
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn description(&self) -> &str {
        &self.description
    }
    
    pub fn initialize(&mut self, config: &CardConfig) -> Result<()> {
        self.config = config.clone();
        Ok(())
    }
    
    pub fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "hello" => {
                println!("Hello from {}!", self.name);
                println!("Arguments: {:?}", args);
                Ok(())
            },
            _ => anyhow::bail!("Unknown command: {}", command),
        }
    }
    
    pub fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "hello".to_string(),
                description: "A simple hello command".to_string(),
                usage: format!("pocket cards run {} hello [args...]", self.name),
            },
        ]
    }
    
    pub fn cleanup(&mut self) -> Result<()> {
        // Clean up any resources
        Ok(())
    }
}

// Export the card creation function
#[no_mangle]
pub extern "C" fn create_card() -> Box<dyn std::any::Any> {
    Box::new(Card::new())
}
"#;
            
            std::fs::write(card_dir.join("src").join("lib.rs"), lib_rs)
                .context("Failed to write lib.rs")?;
            
            // Create a README.md
            let readme = format!(
                r#"# {}

{}

## Installation

This card is installed locally in your Pocket CLI wallet directory.

## Usage

```
pocket cards run {} hello
```

## Commands

- `hello`: A simple hello command

## Development

Edit the `src/lib.rs` file to add your own commands and functionality.
"#,
                name,
                description,
                name
            );
            
            std::fs::write(card_dir.join("README.md"), readme)
                .context("Failed to write README.md")?;
            
            // Register the card in the configuration
            card_manager.register_card_config(name, "local")?;
            
            println!("{} Card created successfully at:", "✅".green());
            println!("  {}", card_dir.display());
            println!("\nTo edit the card, open your code editor at this location.");
            println!("Basic card structure has been created with a template implementation.");
            println!("After editing, build the card with: pocket cards build {}", name);
        },
        Some(CardOperation::Build { name, release }) => {
            println!("{} Building card {}", "🔌".blue(), name.bright_green());
            
            // Check if the card exists
            if !card_manager.card_exists(name) {
                println!("{} Card '{}' not found", "❌".red(), name);
                return Ok(());
            }
            
            // Build the card
            let build_result = std::process::Command::new("cargo")
                .args(&["build", "--release"])
                .current_dir(wallet_dir.join(name))
                .status()
                .context("Failed to execute cargo build command")?;
            
            if !build_result.success() {
                println!("{} Build failed", "❌".red());
                return Ok(());
            }
            
            println!("{} Card built successfully", "✅".green());
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

// Helper function to recursively copy a directory
fn copy_dir_all(src: impl AsRef<std::path::Path>, dst: impl AsRef<std::path::Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    std::fs::create_dir_all(dst)?;
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}
