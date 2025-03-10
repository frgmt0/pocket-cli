mod commands;
mod models;
mod search;
mod storage;
mod utils;
mod version;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(
    name = "pocket",
    about = "A CLI tool for saving, organizing, and retrieving code snippets",
    version = version::VERSION_STRING,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() -> Result<()> {
    let cli = Cli::parse();

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
            println!("\nFor full changelog, see: https://github.com/username/pocket/blob/main/CHANGELOG.md");
        }
        Commands::Edit { id, backpack } => {
            commands::edit_command(id, backpack)?;
        }
    }

    Ok(())
}
