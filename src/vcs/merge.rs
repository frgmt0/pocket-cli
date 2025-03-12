//! Merge functionality for Pocket VCS
//!
//! Handles merging changes between timelines.

use std::path::PathBuf;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use std::fs;
use toml;
use chrono::Utc;
use colored::Colorize;

use crate::vcs::{
    ShoveId, ObjectId, ObjectStore, Tree, TreeEntry,
    Repository, Timeline, Shove, Author
};
use crate::vcs::objects::{EntryType};

/// Strategy to use when merging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// Automatic merge (default)
    Auto,
    /// Fast-forward only (fail if not possible)
    FastForwardOnly,
    /// Always create a merge shove, even for fast-forward
    AlwaysCreateShove,
    /// Resolve conflicts with "ours" strategy
    Ours,
    /// Resolve conflicts with "theirs" strategy
    Theirs,
}

/// Result of a merge operation
#[derive(Debug)]
pub struct MergeResult {
    /// Whether the merge was successful
    pub success: bool,
    
    /// The resulting shove ID (if successful)
    pub shove_id: Option<ShoveId>,
    
    /// Whether the merge was a fast-forward
    pub fast_forward: bool,
    
    /// Conflicts that occurred during the merge
    pub conflicts: Vec<MergeConflict>,
}

/// A conflict that occurred during a merge
#[derive(Debug)]
pub struct MergeConflict {
    /// Path to the conflicted file
    pub path: PathBuf,
    
    /// Base version (common ancestor)
    pub base_id: Option<ObjectId>,
    
    /// "Ours" version
    pub ours_id: Option<ObjectId>,
    
    /// "Theirs" version
    pub theirs_id: Option<ObjectId>,
    
    /// Resolution (if any)
    pub resolution: Option<ConflictResolution>,
}

/// Resolution for a merge conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Use "ours" version
    UseOurs,
    /// Use "theirs" version
    UseTheirs,
    /// Use a custom merged version
    UseMerged(ObjectId),
}

/// Merger for handling merge operations
pub struct Merger<'a> {
    /// Repository to operate on
    pub repo: &'a Repository,
    
    /// Strategy to use for merging
    pub strategy: MergeStrategy,
}

impl<'a> Merger<'a> {
    /// Create a new merger with the default strategy
    pub fn new(repo: &'a Repository) -> Self {
        Self {
            repo,
            strategy: MergeStrategy::Auto,
        }
    }
    
    /// Create a new merger with a custom strategy
    pub fn with_strategy(repo: &'a Repository, strategy: MergeStrategy) -> Self {
        Self { repo, strategy }
    }
    
    /// Merge a timeline into the current timeline
    pub fn merge_timeline(&self, other_timeline: &Timeline) -> Result<MergeResult> {
        // Get the current timeline and head
        let current_timeline = &self.repo.current_timeline;
        let current_head = current_timeline.head.as_ref()
            .ok_or_else(|| anyhow!("Current timeline has no head"))?;
        
        // Get the other timeline's head
        let other_head = other_timeline.head.as_ref()
            .ok_or_else(|| anyhow!("Other timeline has no head"))?;
        
        // If the heads are the same, nothing to do
        if current_head == other_head {
            return Ok(MergeResult {
                success: true,
                shove_id: Some(current_head.clone()),
                fast_forward: true,
                conflicts: vec![],
            });
        }
        
        // Find the common ancestor
        let common_ancestor = self.find_common_ancestor(current_head, other_head)?;
        
        // Check if this is a fast-forward merge
        if let Some(ancestor_id) = &common_ancestor {
            if ancestor_id == current_head {
                // Current is an ancestor of other, so we can fast-forward
                if self.strategy == MergeStrategy::AlwaysCreateShove {
                    // Create a merge shove even though it's a fast-forward
                    return self.create_merge_shove(current_head, other_head, ancestor_id);
                } else {
                    // Just update the current timeline to point to other's head
                    return Ok(MergeResult {
                        success: true,
                        shove_id: Some(other_head.clone()),
                        fast_forward: true,
                        conflicts: vec![],
                    });
                }
            } else if ancestor_id == other_head {
                // Other is an ancestor of current, nothing to do
                return Ok(MergeResult {
                    success: true,
                    shove_id: Some(current_head.clone()),
                    fast_forward: true,
                    conflicts: vec![],
                });
            }
        }
        
        // If we're here, it's not a fast-forward merge
        if self.strategy == MergeStrategy::FastForwardOnly {
            return Ok(MergeResult {
                success: false,
                shove_id: None,
                fast_forward: false,
                conflicts: vec![],
            });
        }
        
        // Perform a three-way merge
        self.three_way_merge(current_head, other_head, common_ancestor.as_ref())
    }
    
    /// Find the common ancestor of two shoves
    fn find_common_ancestor(&self, a: &ShoveId, b: &ShoveId) -> Result<Option<ShoveId>> {
        // Load both shoves
        let a_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", a.as_str()));
        let b_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", b.as_str()));
        
        if !a_path.exists() || !b_path.exists() {
            return Err(anyhow!("One or both shoves not found"));
        }
        
        // Load the shoves
        let a_content = fs::read_to_string(&a_path)?;
        let b_content = fs::read_to_string(&b_path)?;
        
        let a_shove: Shove = toml::from_str(&a_content)?;
        let b_shove: Shove = toml::from_str(&b_content)?;
        
        // If either is an ancestor of the other, return it
        if self.is_ancestor_of(a, b)? {
            return Ok(Some(a.clone()));
        }
        
        if self.is_ancestor_of(b, a)? {
            return Ok(Some(b.clone()));
        }
        
        // Otherwise, find the most recent common ancestor
        // Start with all ancestors of A
        let a_ancestors = self.get_ancestors(a)?;
        
        // For each ancestor of B, check if it's also an ancestor of A
        for b_ancestor in self.get_ancestors(b)? {
            if a_ancestors.contains(&b_ancestor) {
                return Ok(Some(b_ancestor));
            }
        }
        
        // No common ancestor found
        Ok(None)
    }
    
    // Check if a is an ancestor of b
    fn is_ancestor_of(&self, a: &ShoveId, b: &ShoveId) -> Result<bool> {
        if a == b {
            return Ok(true);
        }
        
        // Load b's shove
        let b_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", b.as_str()));
        let b_content = fs::read_to_string(&b_path)?;
        let b_shove: Shove = toml::from_str(&b_content)?;
        
        // Check if a is a direct parent of b
        for parent_id in &b_shove.parent_ids {
            if parent_id == a {
                return Ok(true);
            }
            
            // Recursively check if a is an ancestor of any of b's parents
            if self.is_ancestor_of(a, parent_id)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    // Get all ancestors of a shove
    fn get_ancestors(&self, id: &ShoveId) -> Result<Vec<ShoveId>> {
        let mut ancestors = Vec::new();
        let mut to_process = vec![id.clone()];
        
        while let Some(current_id) = to_process.pop() {
            // Skip if we've already processed this shove
            if ancestors.contains(&current_id) {
                continue;
            }
            
            // Add to ancestors
            ancestors.push(current_id.clone());
            
            // Load the shove
            let shove_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", current_id.as_str()));
            if !shove_path.exists() {
                continue;
            }
            
            let shove_content = fs::read_to_string(&shove_path)?;
            let shove: Shove = toml::from_str(&shove_content)?;
            
            // Add all parents to the processing queue
            for parent_id in shove.parent_ids {
                to_process.push(parent_id);
            }
        }
        
        Ok(ancestors)
    }
    
    /// Perform a three-way merge
    fn three_way_merge(
        &self,
        ours: &ShoveId,
        theirs: &ShoveId,
        base: Option<&ShoveId>,
    ) -> Result<MergeResult> {
        // Get the base shove (common ancestor)
        let base_shove = if let Some(base_id) = base {
            // Load the base shove
            let base_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", base_id.as_str()));
            if base_path.exists() {
                let base_content = fs::read_to_string(&base_path)?;
                Some(toml::from_str::<Shove>(&base_content)?)
            } else {
                return Err(anyhow!("Base shove not found: {}", base_id.as_str()));
            }
        } else {
            None
        };
        
        // Load our shove
        let our_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", ours.as_str()));
        let our_content = fs::read_to_string(&our_path)?;
        let our_shove: Shove = toml::from_str(&our_content)?;
        
        // Load their shove
        let their_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", theirs.as_str()));
        let their_content = fs::read_to_string(&their_path)?;
        let their_shove: Shove = toml::from_str(&their_content)?;
        
        // Get the trees
        let our_tree_path = self.repo.path.join(".pocket").join("objects").join(our_shove.root_tree_id.as_str());
        let our_tree_content = fs::read_to_string(&our_tree_path)?;
        let our_tree: Tree = toml::from_str(&our_tree_content)?;
        
        let their_tree_path = self.repo.path.join(".pocket").join("objects").join(their_shove.root_tree_id.as_str());
        let their_tree_content = fs::read_to_string(&their_tree_path)?;
        let their_tree: Tree = toml::from_str(&their_tree_content)?;
        
        let base_tree = if let Some(base_shove) = &base_shove {
            let base_tree_path = self.repo.path.join(".pocket").join("objects").join(base_shove.root_tree_id.as_str());
            let base_tree_content = fs::read_to_string(&base_tree_path)?;
            Some(toml::from_str::<Tree>(&base_tree_content)?)
        } else {
            None
        };
        
        // Create maps for easier lookup
        let mut our_entries = HashMap::new();
        for entry in our_tree.entries {
            our_entries.insert(entry.name.clone(), entry);
        }
        
        let mut their_entries = HashMap::new();
        for entry in their_tree.entries {
            their_entries.insert(entry.name.clone(), entry);
        }
        
        let base_entries = if let Some(base_tree) = base_tree {
            let mut entries = HashMap::new();
            for entry in base_tree.entries {
                entries.insert(entry.name.clone(), entry);
            }
            Some(entries)
        } else {
            None
        };
        
        // Create a new tree for the merged result
        let mut merged_entries = Vec::new();
        let mut conflicts = Vec::new();
        
        // Process all files in our tree
        for (name, our_entry) in &our_entries {
            if our_entry.entry_type != EntryType::File {
                // Skip non-file entries for simplicity
                merged_entries.push(our_entry.clone());
                continue;
            }
            
            if let Some(their_entry) = their_entries.get(name) {
                // File exists in both trees
                if our_entry.id == their_entry.id {
                    // Same content, no conflict
                    merged_entries.push(our_entry.clone());
                } else {
                    // Different content, check base
                    if let Some(base_entries) = &base_entries {
                        if let Some(base_entry) = base_entries.get(name) {
                            if our_entry.id == base_entry.id {
                                // We didn't change, use theirs
                                merged_entries.push(their_entry.clone());
                            } else if their_entry.id == base_entry.id {
                                // They didn't change, use ours
                                merged_entries.push(our_entry.clone());
                            } else {
                                // Both changed, conflict
                                match self.strategy {
                                    MergeStrategy::Ours => {
                                        // Use our version
                                        merged_entries.push(our_entry.clone());
                                    },
                                    MergeStrategy::Theirs => {
                                        // Use their version
                                        merged_entries.push(their_entry.clone());
                                    },
                                    _ => {
                                        // Create a conflict
                                        conflicts.push(MergeConflict {
                                            path: PathBuf::from(name),
                                            base_id: Some(base_entry.id.clone()),
                                            ours_id: Some(our_entry.id.clone()),
                                            theirs_id: Some(their_entry.id.clone()),
                                            resolution: None,
                                        });
                                        
                                        // For now, use our version
                                        merged_entries.push(our_entry.clone());
                                    }
                                }
                            }
                        } else {
                            // Not in base, both added with different content
                            match self.strategy {
                                MergeStrategy::Ours => {
                                    // Use our version
                                    merged_entries.push(our_entry.clone());
                                },
                                MergeStrategy::Theirs => {
                                    // Use their version
                                    merged_entries.push(their_entry.clone());
                                },
                                _ => {
                                    // Create a conflict
                                    conflicts.push(MergeConflict {
                                        path: PathBuf::from(name),
                                        base_id: None,
                                        ours_id: Some(our_entry.id.clone()),
                                        theirs_id: Some(their_entry.id.clone()),
                                        resolution: None,
                                    });
                                    
                                    // For now, use our version
                                    merged_entries.push(our_entry.clone());
                                }
                            }
                        }
                    } else {
                        // No base, use strategy
                        match self.strategy {
                            MergeStrategy::Ours => {
                                // Use our version
                                merged_entries.push(our_entry.clone());
                            },
                            MergeStrategy::Theirs => {
                                // Use their version
                                merged_entries.push(their_entry.clone());
                            },
                            _ => {
                                // Create a conflict
                                conflicts.push(MergeConflict {
                                    path: PathBuf::from(name),
                                    base_id: None,
                                    ours_id: Some(our_entry.id.clone()),
                                    theirs_id: Some(their_entry.id.clone()),
                                    resolution: None,
                                });
                                
                                // For now, use our version
                                merged_entries.push(our_entry.clone());
                            }
                        }
                    }
                }
            } else {
                // File only in our tree
                if let Some(base_entries) = &base_entries {
                    if let Some(_) = base_entries.get(name) {
                        // In base but not in theirs, they deleted it
                        match self.strategy {
                            MergeStrategy::Ours => {
                                // Keep our version
                                merged_entries.push(our_entry.clone());
                            },
                            MergeStrategy::Theirs => {
                                // They deleted it, so don't include
                            },
                            _ => {
                                // Create a conflict
                                conflicts.push(MergeConflict {
                                    path: PathBuf::from(name),
                                    base_id: Some(base_entries.get(name).unwrap().id.clone()),
                                    ours_id: Some(our_entry.id.clone()),
                                    theirs_id: None,
                                    resolution: None,
                                });
                                
                                // For now, keep our version
                                merged_entries.push(our_entry.clone());
                            }
                        }
                    } else {
                        // Not in base, we added it
                        merged_entries.push(our_entry.clone());
                    }
                } else {
                    // No base, we added it
                    merged_entries.push(our_entry.clone());
                }
            }
        }
        
        // Process files only in their tree
        for (name, their_entry) in &their_entries {
            if their_entry.entry_type != EntryType::File {
                // Skip non-file entries for simplicity
                if !our_entries.contains_key(name) {
                    merged_entries.push(their_entry.clone());
                }
                continue;
            }
            
            if !our_entries.contains_key(name) {
                // File only in their tree
                if let Some(base_entries) = &base_entries {
                    if let Some(_) = base_entries.get(name) {
                        // In base but not in ours, we deleted it
                        match self.strategy {
                            MergeStrategy::Ours => {
                                // We deleted it, so don't include
                            },
                            MergeStrategy::Theirs => {
                                // Keep their version
                                merged_entries.push(their_entry.clone());
                            },
                            _ => {
                                // Create a conflict
                                conflicts.push(MergeConflict {
                                    path: PathBuf::from(name),
                                    base_id: Some(base_entries.get(name).unwrap().id.clone()),
                                    ours_id: None,
                                    theirs_id: Some(their_entry.id.clone()),
                                    resolution: None,
                                });
                                
                                // For now, don't include (follow our deletion)
                            }
                        }
                    } else {
                        // Not in base, they added it
                        merged_entries.push(their_entry.clone());
                    }
                } else {
                    // No base, they added it
                    merged_entries.push(their_entry.clone());
                }
            }
        }
        
        // If there are conflicts and we're not using a strategy that resolves them automatically,
        // return a result with conflicts
        if !conflicts.is_empty() && self.strategy != MergeStrategy::Ours && self.strategy != MergeStrategy::Theirs {
            return Ok(MergeResult {
                success: false,
                shove_id: None,
                fast_forward: false,
                conflicts,
            });
        }
        
        // Create a new tree with the merged entries
        let merged_tree = Tree {
            entries: merged_entries,
        };
        
        // Store the merged tree
        let object_store = ObjectStore::new(self.repo.path.clone());
        let tree_id = object_store.store_tree(&merged_tree)?;
        
        // Create a new shove
        let author = Author {
            name: self.repo.config.user.name.clone(),
            email: self.repo.config.user.email.clone(),
            timestamp: Utc::now(),
        };
        
        let mut parent_ids = vec![ours.clone()];
        if ours != theirs {
            parent_ids.push(theirs.clone());
        }
        
        let message = format!("Merge {} into {}", 
            their_shove.message.lines().next().unwrap_or(""),
            our_shove.message.lines().next().unwrap_or(""));
            
        let shove = Shove::new(&self.repo.pile, parent_ids, author, &message, tree_id);
        
        // Save the shove
        let shove_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", shove.id.as_str()));
        shove.save(&shove_path)?;
        
        Ok(MergeResult {
            success: true,
            shove_id: Some(shove.id.clone()),
            fast_forward: false,
            conflicts: Vec::new(),
        })
    }
    
    /// Create a merge shove
    fn create_merge_shove(
        &self,
        ours: &ShoveId,
        theirs: &ShoveId,
        base: &ShoveId,
    ) -> Result<MergeResult> {
        // Load the shoves
        let ours_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", ours.as_str()));
        let theirs_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", theirs.as_str()));
        
        let ours_content = fs::read_to_string(&ours_path)?;
        let theirs_content = fs::read_to_string(&theirs_path)?;
        
        let our_shove: Shove = toml::from_str(&ours_content)?;
        let their_shove: Shove = toml::from_str(&theirs_content)?;
        
        // Create a new merged tree
        let merged_tree_id = self.merge_trees(&our_shove.root_tree_id, &their_shove.root_tree_id)?;
        
        // Create a new shove with both parents
        let mut parent_ids = vec![our_shove.id.clone()];
        parent_ids.push(their_shove.id.clone());
        
        // Create a merge message
        let message = format!("Merge {} into {}", theirs.as_str(), ours.as_str());
        
        // Create the author information
        let author = Author {
            name: self.repo.config.user.name.clone(),
            email: self.repo.config.user.email.clone(),
            timestamp: Utc::now(),
        };
        
        // Create the new shove
        let new_shove = Shove::new(
            &self.repo.pile,
            parent_ids,
            author,
            &message,
            merged_tree_id,
        );
        
        // Save the shove
        let shove_path = self.repo.path.join(".pocket").join("shoves").join(format!("{}.toml", new_shove.id.as_str()));
        new_shove.save(&shove_path)?;
        
        // Update the current timeline's head
        let timeline_path = self.repo.path.join(".pocket").join("timelines").join("current");
        let mut timeline = Timeline::load(&timeline_path)?;
        timeline.head = Some(new_shove.id.clone());
        timeline.save(&timeline_path)?;
        
        // Return the result
        Ok(MergeResult {
            success: true,
            shove_id: Some(new_shove.id),
            fast_forward: false,
            conflicts: Vec::new(),
        })
    }
    
    fn merge_trees(&self, ours_id: &ObjectId, theirs_id: &ObjectId) -> Result<ObjectId> {
        // Load the trees
        let ours_path = self.repo.path.join(".pocket").join("objects").join(ours_id.as_str());
        let theirs_path = self.repo.path.join(".pocket").join("objects").join(theirs_id.as_str());
        
        let ours_content = fs::read_to_string(&ours_path)?;
        let theirs_content = fs::read_to_string(&theirs_path)?;
        
        let ours_tree: Tree = toml::from_str(&ours_content)?;
        let theirs_tree: Tree = toml::from_str(&theirs_content)?;
        
        // Create a new tree with entries from both
        let mut merged_entries = Vec::new();
        let mut our_entries_map = std::collections::HashMap::new();
        
        // Add all our entries to the map for quick lookup
        for entry in &ours_tree.entries {
            our_entries_map.insert(entry.name.clone(), entry.clone());
        }
        
        // Add all our entries to the merged tree
        for entry in &ours_tree.entries {
            merged_entries.push(entry.clone());
        }
        
        // Add their entries if they don't exist in our tree
        for entry in &theirs_tree.entries {
            if !our_entries_map.contains_key(&entry.name) {
                merged_entries.push(entry.clone());
            }
        }
        
        // Create the new tree
        let merged_tree = Tree {
            entries: merged_entries,
        };
        
        // Save the tree
        let object_store = ObjectStore::new(self.repo.path.clone());
        let tree_id = object_store.store_tree(&merged_tree)?;
        
        Ok(tree_id)
    }
    
    /// Resolve conflicts interactively
    pub fn resolve_conflicts_interactively(&self, conflicts: &[MergeConflict]) -> Result<Vec<ConflictResolution>> {
        let mut resolutions = Vec::new();
        
        println!("\n{} Resolving {} conflicts interactively", "🔄".bright_blue(), conflicts.len());
        
        for (i, conflict) in conflicts.iter().enumerate() {
            println!("\n{} Conflict {}/{}: {}", "⚠️".yellow(), i + 1, conflicts.len(), conflict.path.display());
            
            // Display the conflict
            self.display_conflict(conflict)?;
            
            // Ask for resolution
            let resolution = self.prompt_for_resolution(conflict)?;
            resolutions.push(resolution);
            
            println!("{} Conflict resolved", "✅".green());
        }
        
        println!("\n{} All conflicts resolved", "🎉".green());
        
        Ok(resolutions)
    }
    
    /// Display a conflict to the user
    fn display_conflict(&self, conflict: &MergeConflict) -> Result<()> {
        // Load the base, ours, and theirs content
        let base_content = if let Some(id) = &conflict.base_id {
            self.load_object_content(id)?
        } else {
            String::new()
        };
        
        let ours_content = if let Some(id) = &conflict.ours_id {
            self.load_object_content(id)?
        } else {
            String::new()
        };
        
        let theirs_content = if let Some(id) = &conflict.theirs_id {
            self.load_object_content(id)?
        } else {
            String::new()
        };
        
        // Display the differences
        println!("\n{} Base version:", "⚪".bright_black());
        self.print_content(&base_content, "  ");
        
        println!("\n{} Our version (current timeline):", "🟢".green());
        self.print_content(&ours_content, "  ");
        
        println!("\n{} Their version (incoming timeline):", "🔵".blue());
        self.print_content(&theirs_content, "  ");
        
        Ok(())
    }
    
    /// Print content with line numbers
    fn print_content(&self, content: &str, prefix: &str) {
        for (i, line) in content.lines().enumerate() {
            println!("{}{:3} | {}", prefix, i + 1, line);
        }
        
        if content.is_empty() {
            println!("{}    | (empty file)", prefix);
        }
    }
    
    /// Prompt the user for conflict resolution
    fn prompt_for_resolution(&self, conflict: &MergeConflict) -> Result<ConflictResolution> {
        println!("\nHow would you like to resolve this conflict?");
        println!("  1. {} Use our version (current timeline)", "🟢".green());
        println!("  2. {} Use their version (incoming timeline)", "🔵".blue());
        println!("  3. {} Edit and merge manually", "✏️".yellow());
        
        // In a real implementation, we would use a crate like dialoguer to get user input
        // For now, we'll simulate the user choosing option 1
        println!("\nSelected: 1. Use our version");
        
        if let Some(id) = &conflict.ours_id {
            Ok(ConflictResolution::UseOurs)
        } else {
            // If our version doesn't exist, use theirs
            Ok(ConflictResolution::UseTheirs)
        }
    }
    
    /// Load the content of an object
    fn load_object_content(&self, id: &ObjectId) -> Result<String> {
        let object_path = self.repo.path.join(".pocket").join("objects").join(id.as_str());
        let content = fs::read_to_string(object_path)?;
        Ok(content)
    }
    
    /// Apply conflict resolutions to create a merged tree
    pub fn apply_resolutions(&self, conflicts: &[MergeConflict], resolutions: &[ConflictResolution]) -> Result<Tree> {
        // Create a new tree
        let mut merged_tree = Tree { entries: Vec::new() };
        
        // Apply each resolution
        for (conflict, resolution) in conflicts.iter().zip(resolutions.iter()) {
            match resolution {
                ConflictResolution::UseOurs => {
                    if let Some(id) = &conflict.ours_id {
                        merged_tree.entries.push(TreeEntry {
                            name: conflict.path.to_string_lossy().to_string(),
                            id: id.clone(),
                            entry_type: EntryType::File,
                            permissions: 0o644,
                        });
                    }
                },
                ConflictResolution::UseTheirs => {
                    if let Some(id) = &conflict.theirs_id {
                        merged_tree.entries.push(TreeEntry {
                            name: conflict.path.to_string_lossy().to_string(),
                            id: id.clone(),
                            entry_type: EntryType::File,
                            permissions: 0o644,
                        });
                    }
                },
                ConflictResolution::UseMerged(id) => {
                    merged_tree.entries.push(TreeEntry {
                        name: conflict.path.to_string_lossy().to_string(),
                        id: id.clone(),
                        entry_type: EntryType::File,
                        permissions: 0o644,
                    });
                },
            }
        }
        
        Ok(merged_tree)
    }
} 