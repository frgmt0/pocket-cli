//! Shove (commit) functionality for Pocket VCS
//!
//! Handles the creation and management of shoves (commits).

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use uuid::Uuid;

use crate::vcs::{ObjectId, Pile, FileChange, ChangeType};

/// A unique identifier for a shove
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShoveId(String);

impl ShoveId {
    /// Create a new random shove ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Parse a shove ID from a string
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.to_string()))
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Author information for a shove
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub timestamp: DateTime<Utc>,
}

/// A shove (commit) in the repository
#[derive(Debug, Serialize, Deserialize)]
pub struct Shove {
    /// Unique identifier for this shove
    pub id: ShoveId,
    
    /// Parent shove IDs (multiple for merges)
    pub parent_ids: Vec<ShoveId>,
    
    /// Author information
    pub author: Author,
    
    /// When the shove was created
    pub timestamp: DateTime<Utc>,
    
    /// Commit message
    pub message: String,
    
    /// ID of the root tree object
    pub root_tree_id: ObjectId,
}

impl Shove {
    /// Create a new shove from a pile
    pub fn new(
        pile: &Pile,
        parent_ids: Vec<ShoveId>,
        author: Author,
        message: &str,
        root_tree_id: ObjectId,
    ) -> Self {
        Self {
            id: ShoveId::new(),
            parent_ids,
            author,
            timestamp: Utc::now(),
            message: message.to_string(),
            root_tree_id,
        }
    }
    
    /// Load a shove from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let shove: Self = toml::from_str(&content)?;
        Ok(shove)
    }
    
    /// Save the shove to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Get the changes introduced by this shove
    pub fn get_changes(&self) -> Result<Vec<FileChange>> {
        // This would compare this shove with its parent(s)
        // to determine what files were changed
        
        // Placeholder implementation
        Ok(vec![])
    }
    
    // Additional methods would be implemented here:
    // - get_diff: Get the diff between this shove and another
    // - get_files: Get all files in this shove
    // - etc.
} 