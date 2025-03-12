//! Object storage for Pocket VCS
//!
//! Handles the content-addressable storage of file contents and trees.

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use sha2::{Sha256, Digest};

/// A unique identifier for an object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(String);

impl ObjectId {
    /// Create a new object ID from content
    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hasher.finalize();
        Self(format!("{:x}", hash))
    }
    
    /// Parse an object ID from a string
    pub fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.to_string()))
    }
    
    /// Get the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Type of a tree entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryType {
    File,
    Tree,
}

/// An entry in a tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub id: ObjectId,
    pub entry_type: EntryType,
    pub permissions: u32,
}

/// A tree object representing a directory
#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

/// Object storage for the repository
pub struct ObjectStore {
    base_path: PathBuf,
}

impl ObjectStore {
    /// Create a new object store
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    /// Store a file in the object store
    pub fn store_file(&self, path: &Path) -> Result<ObjectId> {
        let content = fs::read(path)?;
        self.store_object(&content)
    }
    
    /// Store raw content in the object store
    pub fn store_object(&self, content: &[u8]) -> Result<ObjectId> {
        let id = ObjectId::from_content(content);
        let object_path = self.get_object_path(&id);
        
        // Create parent directories if they don't exist
        if let Some(parent) = object_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Only write if the object doesn't already exist
        if !object_path.exists() {
            fs::write(object_path, content)?;
        }
        
        Ok(id)
    }
    
    /// Retrieve an object from the store
    pub fn get_object(&self, id: &ObjectId) -> Result<Vec<u8>> {
        let path = self.get_object_path(id);
        if !path.exists() {
            return Err(anyhow!("Object not found: {}", id.as_str()));
        }
        
        Ok(fs::read(path)?)
    }
    
    /// Check if an object exists in the store
    pub fn has_object(&self, id: &ObjectId) -> bool {
        self.get_object_path(id).exists()
    }
    
    /// Get the path for an object
    fn get_object_path(&self, id: &ObjectId) -> PathBuf {
        let id_str = id.as_str();
        // Use first 2 chars as directory to avoid too many files in one dir
        let prefix = &id_str[0..2];
        let suffix = &id_str[2..];
        self.base_path.join(prefix).join(suffix)
    }
    
    /// Store a tree object
    pub fn store_tree(&self, tree: &Tree) -> Result<ObjectId> {
        let content = toml::to_string(tree)?;
        self.store_object(content.as_bytes())
    }
    
    /// Retrieve a tree object
    pub fn get_tree(&self, id: &ObjectId) -> Result<Tree> {
        let content = self.get_object(id)?;
        let content_str = String::from_utf8(content)?;
        let tree: Tree = toml::from_str(&content_str)?;
        Ok(tree)
    }
    
    // Additional methods would be implemented here:
    // - create_tree_from_directory: Create a tree from a directory
    // - apply_tree_to_directory: Apply a tree to a directory
    // - etc.
} 