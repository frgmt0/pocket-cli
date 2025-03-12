//! Shove (commit) functionality for Pocket VCS
//!
//! Handles the creation and management of shoves (commits).

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use uuid::Uuid;
use std::collections::HashMap;

use crate::vcs::{ObjectId, Pile, FileChange, ChangeType, Repository, Tree};
use crate::vcs::objects::EntryType;

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
    pub fn get_changes(&self, repo: &Repository) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();
        
        // If this is the first shove, all files are considered added
        if self.parent_ids.is_empty() {
            // Get the tree for this shove
            let tree_path = repo.path.join(".pocket").join("objects").join(self.root_tree_id.as_str());
            let tree_content = fs::read_to_string(&tree_path)?;
            let tree: Tree = toml::from_str(&tree_content)?;
            
            // All files in the tree are considered added
            for entry in tree.entries {
                if entry.entry_type == EntryType::File {
                    changes.push(FileChange {
                        path: PathBuf::from(&entry.name),
                        change_type: ChangeType::Added,
                        old_id: None,
                        new_id: Some(entry.id),
                    });
                }
            }
            
            return Ok(changes);
        }
        
        // Get the parent shove
        let parent_id = &self.parent_ids[0]; // For simplicity, just use the first parent
        let parent_path = repo.path.join(".pocket").join("shoves").join(format!("{}.toml", parent_id.as_str()));
        let parent_content = fs::read_to_string(&parent_path)?;
        let parent: Shove = toml::from_str(&parent_content)?;
        
        // Get the trees for both shoves
        let parent_tree_path = repo.path.join(".pocket").join("objects").join(parent.root_tree_id.as_str());
        let current_tree_path = repo.path.join(".pocket").join("objects").join(self.root_tree_id.as_str());
        
        let parent_tree_content = fs::read_to_string(&parent_tree_path)?;
        let current_tree_content = fs::read_to_string(&current_tree_path)?;
        
        let parent_tree: Tree = toml::from_str(&parent_tree_content)?;
        let current_tree: Tree = toml::from_str(&current_tree_content)?;
        
        // Create maps for easier lookup
        let mut parent_entries = HashMap::new();
        for entry in parent_tree.entries {
            parent_entries.insert(entry.name.clone(), entry);
        }
        
        // Find added and modified files
        for entry in &current_tree.entries {
            if entry.entry_type == EntryType::File {
                if let Some(parent_entry) = parent_entries.get(&entry.name) {
                    // File exists in both trees, check if modified
                    if parent_entry.id != entry.id {
                        changes.push(FileChange {
                            path: PathBuf::from(&entry.name),
                            change_type: ChangeType::Modified,
                            old_id: Some(parent_entry.id.clone()),
                            new_id: Some(entry.id.clone()),
                        });
                    }
                } else {
                    // File only exists in current tree, it's added
                    changes.push(FileChange {
                        path: PathBuf::from(&entry.name),
                        change_type: ChangeType::Added,
                        old_id: None,
                        new_id: Some(entry.id.clone()),
                    });
                }
            }
        }
        
        // Find deleted files
        for (name, entry) in parent_entries {
            if entry.entry_type == EntryType::File {
                let exists = current_tree.entries.iter().any(|e| e.name == name);
                if !exists {
                    changes.push(FileChange {
                        path: PathBuf::from(&name),
                        change_type: ChangeType::Deleted,
                        old_id: Some(entry.id),
                        new_id: None,
                    });
                }
            }
        }
        
        Ok(changes)
    }
    
    // Additional methods would be implemented here:
    // - get_diff: Get the diff between this shove and another
    // - get_files: Get all files in this shove
    // - etc.
} 