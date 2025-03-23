use crate::cards::{Card, CardConfig, CardCommand};
use crate::models::{Entry, Backpack};
use crate::storage::StorageManager;
use crate::utils;
use anyhow::{Result, Context, anyhow};
use colored::Colorize;
use std::path::PathBuf;
use std::fs;

/// Card for core commands (search, insert, etc.)
pub struct CoreCard {
    /// Name of the card
    name: String,
    
    /// Version of the card (unused)
    _version: String,
    
    /// Description of the card (unused)
    _description: String,
    
    /// Configuration for the card
    config: CoreCardConfig,
    
    /// Path to the Pocket data directory (kept for future use)
    _data_dir: PathBuf,
}

/// Configuration for the core card
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoreCardConfig {
    /// Maximum number of search results
    pub max_search_results: usize,
    
    /// Default delimiter for inserting content
    pub default_delimiter: String,
}

impl Default for CoreCardConfig {
    fn default() -> Self {
        Self {
            max_search_results: 10,
            default_delimiter: "// --- Pocket CLI Insert ---".to_string(),
        }
    }
}

impl CoreCard {
    /// Creates a new core card
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            name: "core".to_string(),
            _version: env!("CARGO_PKG_VERSION").to_string(),
            _description: "Core card for Pocket CLI".to_string(),
            config: CoreCardConfig::default(),
            _data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Search for entries
    pub fn search(&self, query: &str, limit: usize, backpack: Option<&str>, _exact: bool) -> Result<Vec<Entry>> {
        let storage = StorageManager::new()?;
        
        // For now, we'll use the built-in search, as the API doesn't have exact/semantic differentiation
        let search_results = storage.search_entries(query, backpack, limit)?;
        
        // Return just the entries without content
        Ok(search_results.into_iter().map(|(entry, _)| entry).collect())
    }
    
    /// Insert an entry into a file
    pub fn insert(&self, entry_id: &str, file_path: &str, delimiter: Option<&str>, no_confirm: bool) -> Result<()> {
        let storage = StorageManager::new()?;
        
        // Load the entry and its content
        let (_entry, content) = storage.load_entry(entry_id, None)?;
        
        let delim = delimiter.unwrap_or(&self.config.default_delimiter);
        
        // Read the file content
        let file_content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file {}", file_path))?;
        
        // Get cursor position or end of file
        let cursor_pos = utils::get_cursor_position(&file_content)
            .unwrap_or(file_content.len());
        
        // Insert the content at cursor position
        let new_content = format!(
            "{}\n{}\n{}\n{}",
            &file_content[..cursor_pos],
            delim,
            content,
            &file_content[cursor_pos..]
        );
        
        // Confirm with user if needed
        if !no_confirm {
            println!("Inserting entry {} into {}", entry_id.bold(), file_path.bold());
            let confirm = utils::confirm("Continue?", true)?;
            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        
        // Write the new content
        fs::write(file_path, new_content)
            .with_context(|| format!("Failed to write to file {}", file_path))?;
        
        println!("Successfully inserted entry {} into {}", entry_id.bold(), file_path.bold());
        Ok(())
    }
    
    /// List all entries
    pub fn list(&self, include_backpacks: bool, backpack: Option<&str>, json: bool) -> Result<()> {
        let storage = StorageManager::new()?;
        let entries = storage.list_entries(backpack)?;
        
        if json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
            return Ok(());
        }
        
        if entries.is_empty() {
            println!("No entries found");
            return Ok(());
        }
        
        for entry in entries {
            let backpack_name = if include_backpacks {
                match &entry.source {
                    Some(source) if source.starts_with("backpack:") => {
                        let bp_name = source.strip_prefix("backpack:").unwrap_or("unknown");
                        format!(" [{}]", bp_name.bold())
                    },
                    _ => "".to_string(),
                }
            } else {
                "".to_string()
            };
            
            println!("{}{} - {}", entry.id.bold(), backpack_name, entry.title);
        }
        
        Ok(())
    }
    
    /// Create a new backpack
    pub fn create_backpack(&self, name: &str, description: Option<&str>) -> Result<()> {
        let storage = StorageManager::new()?;
        
        // Create a backpack structure
        let backpack = Backpack {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            created_at: chrono::Utc::now(),
        };
        
        // Save the backpack
        storage.create_backpack(&backpack)?;
        println!("Created backpack: {}", name.bold());
        Ok(())
    }
    
    /// Remove an entry
    pub fn remove(&self, id: &str, force: bool, backpack: Option<&str>) -> Result<()> {
        let storage = StorageManager::new()?;
        
        // Check if entry exists
        let (entry, _) = storage.load_entry(id, backpack)?;
        
        // Confirm with user if not forced
        if !force {
            println!("You are about to remove: {}", id.bold());
            println!("Title: {}", entry.title);
            
            let confirm = utils::confirm("Are you sure?", false)?;
            if !confirm {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        
        // Remove the entry
        storage.remove_entry(id, backpack)?;
        println!("Removed entry: {}", id.bold());
        
        Ok(())
    }
}

impl Card for CoreCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn _description(&self) -> &str {
        "Core card providing essential functions"
    }
    
    fn _initialize(&mut self, config: &CardConfig) -> Result<()> {
        // If there are options in the card config, try to parse them
        if let Some(options_value) = config.options.get("core") {
            if let Ok(options) = serde_json::from_value::<CoreCardConfig>(options_value.clone()) {
                self.config = options;
            }
        }
        
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "search" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing search query"));
                }
                
                let query = &args[0];
                let mut limit = self.config.max_search_results;
                let mut backpack = None;
                let mut exact = false;
                
                // Parse optional arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--limit" => {
                            if i + 1 < args.len() {
                                limit = args[i + 1].parse()?;
                                i += 1;
                            }
                        }
                        "--backpack" => {
                            if i + 1 < args.len() {
                                backpack = Some(args[i + 1].as_str());
                                i += 1;
                            }
                        }
                        "--exact" => {
                            exact = true;
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                let results = self.search(query, limit, backpack, exact)?;
                
                if results.is_empty() {
                    println!("No results found for query: {}", query.bold());
                    return Ok(());
                }
                
                println!("Search results for: {}", query.bold());
                for (i, entry) in results.iter().enumerate() {
                    println!("{}. {} - {}", i + 1, entry.id.bold(), entry.title);
                }
            }
            "insert" => {
                if args.len() < 2 {
                    return Err(anyhow!("Missing entry ID or file path"));
                }
                
                let entry_id = &args[0];
                let file_path = &args[1];
                
                let mut delimiter = None;
                let mut no_confirm = false;
                
                // Parse optional arguments
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--delimiter" => {
                            if i + 1 < args.len() {
                                delimiter = Some(args[i + 1].as_str());
                                i += 1;
                            }
                        }
                        "--no-confirm" => {
                            no_confirm = true;
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                self.insert(entry_id, file_path, delimiter, no_confirm)?;
            }
            "list" => {
                let mut include_backpacks = false;
                let mut backpack = None;
                let mut json = false;
                
                // Parse optional arguments
                let mut i = 0;
                while i < args.len() {
                    match args[i].as_str() {
                        "--include-backpacks" => {
                            include_backpacks = true;
                        }
                        "--backpack" => {
                            if i + 1 < args.len() {
                                backpack = Some(args[i + 1].as_str());
                                i += 1;
                            }
                        }
                        "--json" => {
                            json = true;
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                self.list(include_backpacks, backpack, json)?;
            }
            "create-backpack" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing backpack name"));
                }
                
                let name = &args[0];
                let mut description = None;
                
                // Parse optional arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--description" => {
                            if i + 1 < args.len() {
                                description = Some(args[i + 1].as_str());
                                i += 1;
                            }
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                self.create_backpack(name, description)?;
            }
            "remove" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing entry ID"));
                }
                
                let id = &args[0];
                let mut force = false;
                let mut backpack = None;
                
                // Parse optional arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--force" => {
                            force = true;
                        }
                        "--backpack" => {
                            if i + 1 < args.len() {
                                backpack = Some(args[i + 1].as_str());
                                i += 1;
                            }
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                self.remove(id, force, backpack)?;
            }
            _ => {
                return Err(anyhow!("Unknown command: {}", command));
            }
        }
        
        Ok(())
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "search".to_string(),
                description: "Search for entries".to_string(),
                usage: "search <query> [--limit N] [--backpack NAME] [--exact]".to_string(),
            },
            CardCommand {
                name: "insert".to_string(),
                description: "Insert an entry into a file".to_string(),
                usage: "insert <entry_id> <file_path> [--delimiter TEXT] [--no-confirm]".to_string(),
            },
            CardCommand {
                name: "list".to_string(),
                description: "List all entries".to_string(),
                usage: "list [--include-backpacks] [--backpack NAME] [--json]".to_string(),
            },
            CardCommand {
                name: "create-backpack".to_string(),
                description: "Create a new backpack".to_string(),
                usage: "create-backpack <name> [--description TEXT]".to_string(),
            },
            CardCommand {
                name: "remove".to_string(),
                description: "Remove an entry".to_string(),
                usage: "remove <id> [--force] [--backpack NAME]".to_string(),
            },
        ]
    }
    
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
} 