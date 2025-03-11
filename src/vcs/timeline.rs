//! Timeline (branch) functionality for Pocket VCS
//!
//! Handles the creation and management of timelines (branches).

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use thiserror::Error;

use crate::vcs::ShoveId;

/// Error types specific to timeline operations
#[derive(Error, Debug)]
pub enum TimelineError {
    #[error("Timeline already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Cannot delete the current timeline")]
    CannotDeleteCurrent,
    
    #[error("Timeline has no head shove")]
    NoHead,
}

/// Remote tracking information for a timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTracking {
    pub remote_name: String,
    pub remote_timeline: String,
}

/// A timeline (branch) in the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    /// Name of the timeline
    pub name: String,
    
    /// Current head shove
    pub head: Option<ShoveId>,
    
    /// Remote tracking information (if any)
    pub remote: Option<RemoteTracking>,
}

impl Timeline {
    /// Create a new timeline
    pub fn new(name: &str, head: Option<ShoveId>) -> Self {
        Self {
            name: name.to_string(),
            head,
            remote: None,
        }
    }
    
    /// Load a timeline from a file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let timeline: Self = toml::from_str(&content)?;
        Ok(timeline)
    }
    
    /// Save the timeline to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    /// Update the head of this timeline
    pub fn update_head(&mut self, shove_id: ShoveId) {
        self.head = Some(shove_id);
    }
    
    /// Set up remote tracking for this timeline
    pub fn set_remote_tracking(&mut self, remote_name: &str, remote_timeline: &str) {
        self.remote = Some(RemoteTracking {
            remote_name: remote_name.to_string(),
            remote_timeline: remote_timeline.to_string(),
        });
    }
    
    // Additional methods would be implemented here:
    // - merge: Merge another timeline into this one
    // - is_ancestor: Check if one shove is an ancestor of another
    // - get_common_ancestor: Find the common ancestor of two shoves
    // - etc.
} 