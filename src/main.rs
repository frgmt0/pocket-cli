mod commands;
mod models;
mod search;
mod storage;
mod utils;
mod version;
mod vcs;

use clap::{Parser, Subcommand};
use anyhow::Result;

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

    /// Show timeline graph
    Graph,

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
        Commands::Graph => {
            let path = std::path::Path::new(".");
            vcs::commands::graph_command(path)?;
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
    println!("");
    
    println!("For more information about a specific command, run:");
    println!("    pocket help <COMMAND>");
}
