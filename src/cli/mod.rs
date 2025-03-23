use clap::{Parser, Subcommand, ArgAction};

pub mod handler;

#[derive(Parser)]
#[command(
    name = "pocket",
    about = "A CLI tool for saving, organizing, and retrieving code snippets with integrated version control",
    version = env!("CARGO_PKG_VERSION"),
    author
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, action = ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

        /// Open editor to compose the snippet
        #[arg(short, long)]
        editor: bool,

        /// Store in a specific backpack
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,

        /// Get content from clipboard
        #[arg(long)]
        clipboard: bool,

        /// Generate a summary using LLM
        #[arg(short, long, value_name = "MODEL")]
        summarize: Option<String>,
    },

    #[command(about = "Display all pocket entries")]
    /// List all snippets in your pocket storage
    List {
        /// Display entries from all backpacks
        #[arg(short = 'a', long)]
        all: bool,

        /// Specific backpack to list from
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Limit number of entries to display
        #[arg(short, long, value_name = "N", default_value = "10")]
        limit: usize,
    },

    #[command(about = "Remove an entry from storage")]
    /// Remove a snippet from your pocket storage
    Remove {
        /// ID of the entry to remove
        id: String,

        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,

        /// Backpack the entry is in
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,
    },

    #[command(about = "Create a new backpack for organizing entries")]
    /// Create a new backpack for organizing entries
    Create {
        /// Name of the backpack
        name: String,

        /// Description of the backpack
        #[arg(short, long, value_name = "TEXT")]
        description: Option<String>,
    },

    #[command(about = "Find entries across all backpacks with powerful search algorithms")]
    /// Search for entries in your pocket storage
    Search {
        /// Search query
        query: String,

        /// Maximum results to return
        #[arg(short, long, value_name = "N", default_value = "10")]
        limit: usize,

        /// Search in a specific backpack
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,

        /// Use exact matching instead of semantic search
        #[arg(long)]
        exact: bool,

        /// Search for packages instead of entries
        #[arg(short, long)]
        package: bool,
    },

    #[command(about = "Insert an entry into a file")]
    /// Insert a snippet into a file
    Insert {
        /// ID of the entry to insert
        id: Option<String>,

        /// Path to the file to insert into
        file: Option<String>,

        /// Use the most recent entry
        #[arg(short, long)]
        top: bool,

        /// Don't ask for confirmation
        #[arg(short = 'f', long)]
        no_confirm: bool,

        /// Custom delimiter to use when inserting
        #[arg(short, long, value_name = "TEXT")]
        delimiter: Option<String>,
    },

    #[command(about = "Reload all extensions")]
    /// Reload all extensions and cards
    Reload,

    #[command(about = "Display help information")]
    /// Show help information for commands and extensions
    ShowHelp {
        /// Command to show help for
        command: Option<String>,

        /// List all available extensions
        #[arg(short, long)]
        extensions: bool,
    },

    #[command(about = "Create and execute command chains")]
    /// Lint code before adding to pocket storage
    Lint {
        /// Optional workflow to run
        workflow: Option<String>,
    },

    #[command(about = "Remove a saved workflow")]
    /// Delete a saved workflow
    DeleteWorkflow {
        /// Name of the workflow to delete
        name: String,
    },

    #[command(about = "Display version information")]
    /// Show version information
    Version,

    #[command(about = "Edit an existing entry")]
    /// Edit a snippet in your pocket storage
    Edit {
        /// ID of the entry to edit
        id: String,

        /// Don't ask for confirmation before saving
        #[arg(short, long)]
        force: bool,

        /// Backpack the entry is in
        #[arg(short, long, value_name = "NAME")]
        backpack: Option<String>,
    },

    #[command(about = "Execute a script")]
    /// Execute a saved script
    Execute {
        /// Name of the script to execute
        name: String,

        /// Arguments to pass to the script
        args: Vec<String>,
    },

    #[command(about = "🔌 Manage cards")]
    /// Manage cards (extensions)
    Cards {
        #[command(subcommand)]
        operation: Option<CardOperation>,
    },

    #[command(about = "🧪 Blend shell scripts into your shell configuration")]
    /// Blend shell scripts into your shell environment
    Blend {
        /// Path to shell script file to blend into shell configuration
        script_file: Option<String>,

        /// Create as an executable hook command (accessible with @name)
        #[arg(short, long)]
        executable: bool,

        #[command(subcommand)]
        command: Option<BlendCommands>,
    },
}

#[derive(Subcommand)]
pub enum CardOperation {
    /// List all available cards
    List {
        /// Show detailed information
        #[arg(short, long)]
        detail: bool,
    },

    /// Enable a card
    Enable {
        /// Name of the card to enable
        name: String,
    },

    /// Disable a card
    Disable {
        /// Name of the card to disable
        name: String,
    },

    /// Add a new card
    Add {
        /// Name of the card
        name: String,

        /// URL of the card repository
        url: String,
    },

    /// Remove a card
    Remove {
        /// Name of the card to remove
        name: String,

        /// Don't ask for confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Build a card
    Build {
        /// Name of the card to build
        name: String,

        /// Create a release build
        #[arg(short, long)]
        release: bool,
    },
    
    /// Create a new card template
    Create {
        /// Name of the card to create
        name: String,
        
        /// Description of the card
        #[arg(short, long)]
        description: String,
    },
}

#[derive(Subcommand)]
pub enum BlendCommands {
    /// Edit an existing hook
    Edit {
        /// Name of the hook to edit (with or without @ prefix)
        hook_name: String,
    },

    /// List all installed hooks
    List,

    /// Run a hook command directly
    Run {
        /// Name of the hook to run (with or without @ prefix)
        hook_name: String,

        /// Arguments to pass to the hook
        args: Vec<String>,
    },
} 