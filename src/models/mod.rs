use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use std::collections::HashMap;

/// Represents an entry in the pocket storage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    /// Unique identifier for the entry
    pub id: String,
    
    /// Title or first line of the entry
    pub title: String,
    
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    
    /// When the entry was last updated
    pub updated_at: DateTime<Utc>,
    
    /// Source of the entry (file path, etc.)
    pub source: Option<String>,
    
    /// Tags associated with the entry
    pub tags: Vec<String>,
    
    /// Type of content (code, text, etc.)
    pub content_type: ContentType,
    
    /// Metadata associated with the entry
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Represents the type of content in an entry
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContentType {
    Code,
    Text,
    Script,
    Other(String),
}

/// Represents a backpack for organizing entries
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Backpack {
    /// Name of the backpack
    pub name: String,
    
    /// Description of the backpack
    pub description: Option<String>,
    
    /// When the backpack was created
    pub created_at: DateTime<Utc>,
}

/// Represents a saved workflow
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Workflow {
    /// Name of the workflow
    pub name: String,
    
    /// Commands in the workflow
    pub commands: Vec<WorkflowCommand>,
    
    /// When the workflow was created
    pub created_at: DateTime<Utc>,
}

/// Represents a command in a workflow
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowCommand {
    /// The command to execute
    pub command: String,
    
    /// Arguments for the command
    pub args: Vec<String>,
}

impl Entry {
    /// Create a new entry
    pub fn new(title: String, content_type: ContentType, source: Option<String>, tags: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            created_at: now,
            updated_at: now,
            source,
            tags,
            content_type,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the entry
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
    
    /// Get metadata from the entry
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }
}

impl Backpack {
    /// Create a new backpack
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            name,
            description,
            created_at: Utc::now(),
        }
    }
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: String, commands: Vec<WorkflowCommand>) -> Self {
        Self {
            name,
            commands,
            created_at: Utc::now(),
        }
    }
}

impl WorkflowCommand {
    /// Parse a command string into a WorkflowCommand
    pub fn parse(command_str: &str) -> Result<Self> {
        let command_str = command_str.trim();
        if command_str.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        
        let parts: Vec<&str> = command_str.split_whitespace().collect();
        
        Ok(Self {
            command: parts[0].to_string(),
            args: parts[1..].iter().map(|s| s.to_string()).collect(),
        })
    }
}

/// Configuration for the pocket application
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// User preferences
    pub user: UserConfig,
    
    /// Display settings
    pub display: DisplayConfig,
    
    /// Search settings
    pub search: SearchConfig,
    
    /// Extension settings
    pub extensions: ExtensionConfig,
}

/// User configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    /// Default editor for -e flag
    pub editor: String,
    
    /// Default backpack for new entries
    pub default_backpack: String,
}

/// Display configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Enable colorful output
    pub color: bool,
    
    /// Tree style (unicode, ascii, or minimal)
    pub tree_style: TreeStyle,
}

/// Search configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Search algorithm (semantic or literal)
    pub algorithm: SearchAlgorithm,
    
    /// Maximum number of search results
    pub max_results: usize,
}

/// Extension configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// Auto-reload extensions when they change
    pub auto_reload: bool,
}

/// Tree style for display
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TreeStyle {
    Unicode,
    Ascii,
    Minimal,
}

/// Search algorithm
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SearchAlgorithm {
    Semantic,
    Literal,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user: UserConfig {
                editor: "vim".to_string(),
                default_backpack: "general".to_string(),
            },
            display: DisplayConfig {
                color: true,
                tree_style: TreeStyle::Unicode,
            },
            search: SearchConfig {
                algorithm: SearchAlgorithm::Semantic,
                max_results: 10,
            },
            extensions: ExtensionConfig {
                auto_reload: true,
            },
        }
    }
} 