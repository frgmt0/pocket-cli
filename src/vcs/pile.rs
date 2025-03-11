//! Pile (staging area) functionality for Pocket VCS
//!
//! Handles the staging of changes before they are committed.

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};

use crate::vcs::{ObjectId, ObjectStore, ShoveId};

/// Status of a file in the pile
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PileStatus {
    Added,
    Modified,
    Deleted,
    Renamed(PathBuf),
}

/// An entry in the pile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PileEntry {
    pub status: PileStatus,
    pub object_id: ObjectId,
    pub original_path: PathBuf,
}

/// The pile (staging area)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pile {
    /// The base shove this pile is built on
    pub base_shove: Option<ShoveId>,
    
    /// Entries in the pile, keyed by path
    pub entries: HashMap<PathBuf, PileEntry>,
}

impl Pile {
    /// Create a new empty pile
    pub fn new() -> Self {
        Self {
            base_shove: None,
            entries: HashMap::new(),
        }
    }
    
    /// Load a pile from a file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        
        let content = fs::read_to_string(path)?;
        let pile: Self = toml::from_str(&content)?;
        Ok(pile)
    }
    
    /// Save the pile to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Add a file to the pile
    pub fn add_path(&mut self, path: &Path, object_store: &ObjectStore) -> Result<()> {
        // Store the file content in the object store
        let object_id = object_store.store_file(path)?;
        
        // Add the file to the pile
        let entry = PileEntry {
            status: PileStatus::Added, // This would be determined based on repo state
            object_id,
            original_path: path.to_path_buf(),
        };
        
        self.entries.insert(path.to_path_buf(), entry);
        Ok(())
    }
    
    /// Remove a file from the pile
    pub fn remove_path(&mut self, path: &Path) -> Result<()> {
        if !self.entries.contains_key(path) {
            return Err(anyhow!("Path not in pile: {}", path.display()));
        }
        
        self.entries.remove(path);
        Ok(())
    }
    
    /// Clear the pile
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        Ok(())
    }
    
    /// Check if the pile is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Get the number of entries in the pile
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    // Additional methods would be implemented here:
    // - add_all: Add all changes to the pile
    // - add_pattern: Add files matching a pattern
    // - add_interactive: Interactive adding
    // - etc.
} 