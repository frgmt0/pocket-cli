//! Diff functionality for Pocket VCS
//!
//! Handles the calculation and representation of differences between files.

use std::path::{Path, PathBuf};
use anyhow::Result;

use crate::vcs::{ObjectId, ObjectStore};

/// Options for controlling diff behavior
#[derive(Debug, Clone)]
pub struct DiffOptions {
    pub context_lines: usize,
    pub ignore_whitespace: bool,
    pub ignore_case: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            context_lines: 3,
            ignore_whitespace: false,
            ignore_case: false,
        }
    }
}

/// A single change in a diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffChange {
    /// Lines added (new content)
    Added {
        start: usize,
        lines: Vec<String>,
    },
    /// Lines removed (old content)
    Removed {
        start: usize,
        lines: Vec<String>,
    },
    /// Lines changed (old content to new content)
    Changed {
        old_start: usize,
        old_lines: Vec<String>,
        new_start: usize,
        new_lines: Vec<String>,
    },
}

/// A hunk of changes in a diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub changes: Vec<DiffChange>,
}

/// The result of a diff operation
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub hunks: Vec<DiffHunk>,
    pub is_binary: bool,
}

/// Diff calculator
pub struct Diff {
    pub options: DiffOptions,
}

impl Diff {
    /// Create a new diff calculator with default options
    pub fn new() -> Self {
        Self {
            options: DiffOptions::default(),
        }
    }
    
    /// Create a new diff calculator with custom options
    pub fn with_options(options: DiffOptions) -> Self {
        Self { options }
    }
    
    /// Calculate the diff between two files
    pub fn diff_files(&self, old_path: &Path, new_path: &Path) -> Result<DiffResult> {
        // Read file contents
        let old_content = std::fs::read_to_string(old_path)?;
        let new_content = std::fs::read_to_string(new_path)?;
        
        // Calculate diff
        self.diff_content(
            old_path.to_path_buf(),
            &old_content,
            new_path.to_path_buf(),
            &new_content,
        )
    }
    
    /// Calculate the diff between two objects in the object store
    pub fn diff_objects(
        &self,
        object_store: &ObjectStore,
        old_id: &ObjectId,
        old_path: PathBuf,
        new_id: &ObjectId,
        new_path: PathBuf,
    ) -> Result<DiffResult> {
        // Get object contents
        let old_content = String::from_utf8(object_store.get_object(old_id)?)?;
        let new_content = String::from_utf8(object_store.get_object(new_id)?)?;
        
        // Calculate diff
        self.diff_content(old_path, &old_content, new_path, &new_content)
    }
    
    /// Calculate the diff between two content strings
    pub fn diff_content(
        &self,
        old_path: PathBuf,
        old_content: &str,
        new_path: PathBuf,
        new_content: &str,
    ) -> Result<DiffResult> {
        // Split content into lines
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();
        
        // Check if files are binary
        let is_binary = old_content.contains('\0') || new_content.contains('\0');
        if is_binary {
            return Ok(DiffResult {
                old_path,
                new_path,
                hunks: vec![],
                is_binary: true,
            });
        }
        
        // Calculate diff using the Myers diff algorithm
        // This is a simplified implementation - a real one would use a more
        // efficient algorithm like the one in the `diff` crate
        let hunks = self.myers_diff(&old_lines, &new_lines)?;
        
        Ok(DiffResult {
            old_path,
            new_path,
            hunks,
            is_binary: false,
        })
    }
    
    /// Implement the Myers diff algorithm
    fn myers_diff(&self, old_lines: &[&str], new_lines: &[&str]) -> Result<Vec<DiffHunk>> {
        // This is a placeholder for the actual implementation
        // A real implementation would use a proper diff algorithm
        
        // For now, just return a simple diff that shows all lines as changed
        let mut hunks = Vec::new();
        
        if !old_lines.is_empty() || !new_lines.is_empty() {
            let changes = vec![DiffChange::Changed {
                old_start: 1,
                old_lines: old_lines.iter().map(|s| s.to_string()).collect(),
                new_start: 1,
                new_lines: new_lines.iter().map(|s| s.to_string()).collect(),
            }];
            
            hunks.push(DiffHunk {
                old_start: 1,
                old_count: old_lines.len(),
                new_start: 1,
                new_count: new_lines.len(),
                changes,
            });
        }
        
        Ok(hunks)
    }
    
    // Additional methods would be implemented here:
    // - format_diff: Format a diff result as a string
    // - apply_diff: Apply a diff to a file
    // - etc.
} 