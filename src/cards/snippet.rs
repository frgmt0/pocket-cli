use crate::cards::{Card, CardConfig, CardCommand};
use crate::utils::{read_clipboard, summarize_text, SummaryMetadata};
use crate::models::Entry;
use crate::storage::StorageManager;
use anyhow::{Result, anyhow, Context};
use std::path::PathBuf;
use std::fs;

/// Card for enhanced snippet functionality
pub struct SnippetCard {
    /// Name of the card
    name: String,
    
    /// Version of the card
    version: String,
    
    /// Description of the card
    description: String,
    
    /// Configuration for the card
    config: SnippetCardConfig,
    
    /// Path to the Pocket data directory (kept for future use)
    _data_dir: PathBuf,
}

/// Configuration for the snippet card
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnippetCardConfig {
    /// Whether to automatically summarize snippets
    pub auto_summarize: bool,
    
    /// Maximum length for auto-generated summaries
    pub max_summary_length: usize,
    
    /// Whether to include summaries in search results
    pub search_in_summaries: bool,
    
    /// Weight to give summary matches in search results (0.0-1.0)
    pub summary_search_weight: f32,
}

impl Default for SnippetCardConfig {
    fn default() -> Self {
        Self {
            auto_summarize: true,
            max_summary_length: 150,
            search_in_summaries: true,
            summary_search_weight: 0.7,
        }
    }
}

impl SnippetCard {
    /// Creates a new snippet card
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            name: "snippet".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Enhanced snippet functionality with clipboard and summarization features".to_string(),
            config: SnippetCardConfig::default(),
            _data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Adds a snippet from a file or editor
    pub fn add(&self, 
              file: Option<&str>,
              message: Option<&str>,
              use_editor: bool, 
              use_clipboard: bool,
              backpack: Option<&str>,
              summarize: Option<&str>) -> Result<String> {
        // Initialize content
        let content = if let Some(file_path) = file {
            // Read from file
            fs::read_to_string(file_path)
                .context(format!("Failed to read file: {}", file_path))?
        } else if use_editor {
            // Open editor
            crate::utils::open_editor(None)
                .context("Failed to open editor")?
        } else if use_clipboard {
            // Read from clipboard
            read_clipboard()
                .context("Failed to read from clipboard")?
        } else {
            // No content source provided
            return Err(anyhow!("No content source provided. Use --file, --editor, or --clipboard options"));
        };

        if content.trim().is_empty() {
            return Err(anyhow!("Content is empty"));
        }
        
        // Detect content type
        let content_type = if let Some(file_path) = file {
            let path = PathBuf::from(file_path);
            crate::utils::detect_content_type(Some(&path), Some(&content))
        } else {
            crate::utils::detect_content_type(None, Some(&content))
        };
        
        // Create a title from message, first line, or first 50 chars if no lines
        let title = if let Some(msg) = message {
            msg.to_string()
        } else {
            content.lines().next()
                .unwrap_or(&content[..std::cmp::min(50, content.len())])
                .to_string()
        };
        
        // Create entry
        let mut entry = Entry::new(title, content_type, None, vec![]);
        
        // Create summary metadata
        let summary = if let Some(manual_summary) = summarize {
            // User provided a summary, use it
            SummaryMetadata::new(manual_summary.to_string(), false)
        } else if self.config.auto_summarize {
            // Auto-generate a summary
            let summary = summarize_text(&content)
                .unwrap_or_else(|_| {
                    // Fallback: use first line or first 100 chars
                    content.lines().next()
                        .unwrap_or(&content[..std::cmp::min(100, content.len())])
                        .to_string()
                });
                
            // Truncate if needed
            let summary = if summary.len() > self.config.max_summary_length {
                format!("{}...", &summary[..self.config.max_summary_length - 3])
            } else {
                summary
            };
            
            SummaryMetadata::new(summary, true)
        } else {
            // No summarization requested
            SummaryMetadata::new("".to_string(), true)
        };
        
        // Add summary metadata to entry
        entry.add_metadata("summary", &summary.to_json());
        
        // Save the entry
        let storage = StorageManager::new()?;
        storage.save_entry(&entry, &content, backpack)?;
        
        Ok(entry.id)
    }
    
    /// Adds a snippet from clipboard content
    pub fn add_from_clipboard(&self, 
                              user_summary: Option<&str>, 
                              backpack: Option<&str>) -> Result<String> {
        // Read content from clipboard
        let content = read_clipboard()
            .context("Failed to read from clipboard")?;
            
        if content.trim().is_empty() {
            return Err(anyhow!("Clipboard is empty"));
        }
        
        // Detect content type
        let content_type = crate::utils::detect_content_type(None, Some(&content));
        
        // Create a title from the first line, or first 50 chars if no lines
        let title = content.lines().next()
            .unwrap_or(&content[..std::cmp::min(50, content.len())])
            .to_string();
        
        // Create entry
        let mut entry = Entry::new(title, content_type, None, vec![]);
        
        // Create summary metadata
        let summary = if let Some(manual_summary) = user_summary {
            // User provided a summary, use it
            SummaryMetadata::new(manual_summary.to_string(), false)
        } else if self.config.auto_summarize {
            // Auto-generate a summary
            let summary = summarize_text(&content)
                .unwrap_or_else(|_| {
                    // Fallback: use first line or first 100 chars
                    content.lines().next()
                        .unwrap_or(&content[..std::cmp::min(100, content.len())])
                        .to_string()
                });
                
            // Truncate if needed
            let summary = if summary.len() > self.config.max_summary_length {
                format!("{}...", &summary[..self.config.max_summary_length - 3])
            } else {
                summary
            };
            
            SummaryMetadata::new(summary, true)
        } else {
            // No summarization requested
            SummaryMetadata::new("".to_string(), true)
        };
        
        // Add summary metadata to entry
        entry.add_metadata("summary", &summary.to_json());
        
        // Save the entry
        let storage = StorageManager::new()?;
        storage.save_entry(&entry, &content, backpack)?;
        
        Ok(entry.id)
    }
    
    /// Searches for snippets, including in summaries if configured
    pub fn search(&self, query: &str, limit: usize, backpack: Option<&str>) -> Result<Vec<(Entry, String, Option<SummaryMetadata>)>> {
        let storage = StorageManager::new()?;
        
        // Basic search first
        let mut results = Vec::new();
        let entries = storage.search_entries(query, backpack, limit)?;
        
        for (entry, content) in entries {
            // Load summary metadata if it exists
            let summary = if let Some(summary_json) = entry.get_metadata("summary") {
                match SummaryMetadata::from_json(summary_json) {
                    Ok(summary) => Some(summary),
                    Err(_) => None,
                }
            } else {
                None
            };
            
            results.push((entry, content, summary));
        }
        
        // If searching in summaries is enabled, also search in summaries
        if self.config.search_in_summaries {
            let all_entries = storage.list_entries(backpack)?;
            
            for entry in all_entries {
                // Skip entries already in results
                if results.iter().any(|(e, _, _)| e.id == entry.id) {
                    continue;
                }
                
                // Get summary metadata
                if let Some(summary_json) = entry.get_metadata("summary") {
                    if let Ok(summary) = SummaryMetadata::from_json(summary_json) {
                        // Check if query matches summary
                        if summary.summary.to_lowercase().contains(&query.to_lowercase()) {
                            // Load the entry content
                            if let Ok((entry, content)) = storage.load_entry(&entry.id, backpack) {
                                results.push((entry, content, Some(summary)));
                                
                                // Check if we've reached the limit
                                if results.len() >= limit {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}

impl Card for SnippetCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn _description(&self) -> &str {
        &self.description
    }
    
    fn _initialize(&mut self, config: &CardConfig) -> Result<()> {
        // If there are options in the card config, try to parse them
        if let Some(options_value) = config.options.get("snippet") {
            if let Ok(options) = serde_json::from_value::<SnippetCardConfig>(options_value.clone()) {
                self.config = options;
            }
        }
        
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "add" => {
                let mut file = None;
                let mut message = None;
                let mut use_editor = false;
                let mut use_clipboard = false;
                let mut backpack = None;
                let mut summarize = None;
                
                // Parse arguments
                let mut i = 0;
                while i < args.len() {
                    if args[i].starts_with("--file=") {
                        file = Some(args[i][7..].to_string());
                        i += 1;
                    } else if args[i] == "--file" {
                        if i + 1 < args.len() {
                            file = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err(anyhow!("--file requires a file path"));
                        }
                    } else if args[i].starts_with("--message=") {
                        message = Some(args[i][10..].to_string());
                        i += 1;
                    } else if args[i] == "--message" {
                        if i + 1 < args.len() {
                            message = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err(anyhow!("--message requires a message string"));
                        }
                    } else if args[i] == "--editor" {
                        use_editor = true;
                        i += 1;
                    } else if args[i] == "--clipboard" {
                        use_clipboard = true;
                        i += 1;
                    } else if args[i].starts_with("--backpack=") {
                        backpack = Some(args[i][11..].to_string());
                        i += 1;
                    } else if args[i] == "--backpack" {
                        if i + 1 < args.len() {
                            backpack = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err(anyhow!("--backpack requires a backpack name"));
                        }
                    } else if args[i].starts_with("--summarize=") {
                        summarize = Some(args[i][12..].to_string());
                        i += 1;
                    } else if args[i] == "--summarize" {
                        if i + 1 < args.len() {
                            summarize = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            return Err(anyhow!("--summarize requires a summary string"));
                        }
                    } else {
                        i += 1;
                    }
                }
                
                // Add snippet
                let id = self.add(file.as_deref(), message.as_deref(), use_editor, use_clipboard, backpack.as_deref(), summarize.as_deref())?;
                println!("Added snippet with ID: {}", id);
                Ok(())
            },
            "add-from-clipboard" => {
                let mut user_summary = None;
                let mut backpack = None;
                
                // Parse arguments
                let mut i = 0;
                while i < args.len() {
                    match args[i].as_str() {
                        "--summarize" => {
                            if i + 1 < args.len() {
                                user_summary = Some(args[i + 1].as_str());
                                i += 2;
                            } else {
                                return Err(anyhow!("--summarize requires a summary string"));
                            }
                        },
                        "--backpack" => {
                            if i + 1 < args.len() {
                                backpack = Some(args[i + 1].as_str());
                                i += 2;
                            } else {
                                return Err(anyhow!("--backpack requires a backpack name"));
                            }
                        },
                        _ => {
                            i += 1;
                        }
                    }
                }
                
                // Add from clipboard
                let id = self.add_from_clipboard(user_summary, backpack)?;
                println!("Added snippet from clipboard with ID: {}", id);
                Ok(())
            },
            "search" => {
                if args.is_empty() {
                    return Err(anyhow!("search requires a query string"));
                }
                
                let query = &args[0];
                let limit = if args.len() > 1 {
                    args[1].parse().unwrap_or(10)
                } else {
                    10
                };
                
                let mut backpack = None;
                let mut i = 2;
                while i < args.len() {
                    match args[i].as_str() {
                        "--backpack" => {
                            if i + 1 < args.len() {
                                backpack = Some(args[i + 1].as_str());
                                i += 2;
                            } else {
                                return Err(anyhow!("--backpack requires a backpack name"));
                            }
                        },
                        _ => {
                            i += 1;
                        }
                    }
                }
                
                // Search
                let results = self.search(query, limit, backpack)?;
                
                if results.is_empty() {
                    println!("No results found");
                    return Ok(());
                }
                
                println!("Search results for '{}':", query);
                for (i, (entry, content, summary)) in results.iter().enumerate() {
                    println!("{}. {} ({})", i + 1, entry.title, entry.id);
                    
                    // Show summary if available
                    if let Some(summary) = summary {
                        println!("   Summary: {}", summary.summary);
                    }
                    
                    // Show snippet of content
                    let preview = if content.len() > 100 {
                        format!("{}...", &content[..97])
                    } else {
                        content.clone()
                    };
                    println!("   Content: {}", preview.replace('\n', " "));
                    println!();
                }
                
                Ok(())
            },
            "config" => {
                // Show current configuration
                println!("Snippet card configuration:");
                println!("  Auto-summarize: {}", self.config.auto_summarize);
                println!("  Max summary length: {}", self.config.max_summary_length);
                println!("  Search in summaries: {}", self.config.search_in_summaries);
                println!("  Summary search weight: {}", self.config.summary_search_weight);
                Ok(())
            },
            _ => Err(anyhow!("Unknown command: {}", command))
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "add".to_string(),
                description: "Add a new snippet from a file or editor".to_string(),
                usage: "pocket cards execute snippet add [--file=FILE] [--message=MESSAGE] [--editor] [--backpack=BACKPACK] [--summarize=SUMMARY]".to_string(),
            },
            CardCommand {
                name: "add-from-clipboard".to_string(),
                description: "Add a snippet from clipboard content".to_string(),
                usage: "pocket cards execute snippet add-from-clipboard [--summarize SUMMARY] [--backpack BACKPACK]".to_string(),
            },
            CardCommand {
                name: "search".to_string(),
                description: "Search for snippets, including in summaries".to_string(),
                usage: "pocket cards execute snippet search QUERY [LIMIT] [--backpack BACKPACK]".to_string(),
            },
            CardCommand {
                name: "config".to_string(),
                description: "Show current snippet card configuration".to_string(),
                usage: "pocket cards execute snippet config".to_string(),
            },
        ]
    }
    
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
} 