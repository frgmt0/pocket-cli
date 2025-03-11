//! Pocket Version Control System
//! 
//! A custom VCS implementation that provides intuitive version control
//! with a focus on user experience and modern workflows.

// Module declarations
pub mod repository;
pub mod pile;
pub mod shove;
pub mod timeline;
pub mod objects;
pub mod diff;
pub mod merge;
pub mod remote;
pub mod commands;

// Re-export the main types for easier access
pub use repository::{Repository, RepositoryError};
pub use pile::{Pile, PileEntry, PileStatus};
pub use shove::{Shove, ShoveId, Author};
pub use timeline::{Timeline, TimelineError};
pub use objects::{ObjectStore, ObjectId, Tree, TreeEntry};
pub use diff::{Diff, DiffResult, DiffOptions};
pub use merge::{MergeResult, MergeStrategy, ConflictResolution};
pub use remote::{Remote, RemoteError, RemoteTracking};

// Common types used throughout the VCS module
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use thiserror::Error;
use anyhow::{Result, anyhow};

/// Common error types for VCS operations
#[derive(Error, Debug)]
pub enum VcsError {
    #[error("Repository not found at {0}")]
    RepositoryNotFound(PathBuf),
    
    #[error("Invalid repository state: {0}")]
    InvalidRepositoryState(String),
    
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
    
    #[error("Timeline not found: {0}")]
    TimelineNotFound(String),
    
    #[error("Shove not found: {0}")]
    ShoveNotFound(String),
    
    #[error("Merge conflict in {0}")]
    MergeConflict(PathBuf),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Remote error: {0}")]
    RemoteError(String),
}

/// Status of the repository
#[derive(Debug, Clone)]
pub struct RepoStatus {
    pub current_timeline: String,
    pub head_shove: Option<ShoveId>,
    pub piled_files: Vec<PileEntry>,
    pub modified_files: Vec<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
    pub conflicts: Vec<PathBuf>,
}

/// A change to a file
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub old_id: Option<ObjectId>,
    pub new_id: Option<ObjectId>,
}

/// Type of change to a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed(PathBuf), // Contains the old path
    Copied(PathBuf),  // Contains the source path
}

// Initialize the VCS system
pub fn init() -> Result<()> {
    // Any global initialization needed for the VCS system
    Ok(())
} 