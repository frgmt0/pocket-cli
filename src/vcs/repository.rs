//! Repository management for Pocket VCS
//!
//! Handles creation, opening, and basic operations on repositories.

use std::path::{Path, PathBuf};
use std::fs;
use chrono::Utc;
use serde::{Serialize, Deserialize};
use anyhow::{Result, anyhow};
use thiserror::Error;
use walkdir;
use glob;

use crate::vcs::{
    ObjectId, ObjectStore, ShoveId,
    RepoStatus,
    objects::{Tree, TreeEntry, EntryType},
    shove::Shove,
    Author,
};
use crate::vcs::pile::Pile;
use crate::vcs::timeline::Timeline;

/// Error types specific to repository operations
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Repository already exists at {0}")]
    AlreadyExists(PathBuf),
    
    #[error("Not a valid Pocket repository: {0}")]
    NotARepository(PathBuf),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Repository configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub user: UserConfig,
    pub core: CoreConfig,
    pub remote: RemoteConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreConfig {
    pub default_timeline: String,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteConfig {
    pub default_remote: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user: UserConfig {
                name: "Unknown User".to_string(),
                email: "user@example.com".to_string(),
            },
            core: CoreConfig {
                default_timeline: "main".to_string(),
                ignore_patterns: vec![".DS_Store".to_string(), "*.log".to_string()],
            },
            remote: RemoteConfig {
                default_remote: None,
            },
        }
    }
}

/// Represents a Pocket VCS repository
pub struct Repository {
    /// Path to the repository root
    pub path: PathBuf,
    
    /// Repository configuration
    pub config: Config,
    
    /// Current timeline (branch)
    pub current_timeline: Timeline,
    
    /// Current pile (staging area)
    pub pile: Pile,
    
    /// Object storage
    pub object_store: ObjectStore,
}

impl Repository {
    /// Create a new repository at the given path
    pub fn new(path: &Path) -> Result<Self> {
        let repo_path = path.join(".pocket");
        
        // Check if repository already exists
        if repo_path.exists() {
            return Err(RepositoryError::AlreadyExists(repo_path).into());
        }
        
        // Create repository directory structure
        fs::create_dir_all(&repo_path)?;
        fs::create_dir_all(repo_path.join("objects"))?;
        fs::create_dir_all(repo_path.join("shoves"))?;
        fs::create_dir_all(repo_path.join("timelines"))?;
        fs::create_dir_all(repo_path.join("piles"))?;
        fs::create_dir_all(repo_path.join("snapshots"))?;
        
        // Create default configuration
        let config = Config::default();
        let config_path = repo_path.join("config.toml");
        let config_str = toml::to_string_pretty(&config)?;
        fs::write(config_path, config_str)?;
        
        // Create initial timeline (main)
        let timeline_path = repo_path.join("timelines").join("main.toml");
        let timeline = Timeline::new("main", None);
        let timeline_str = toml::to_string_pretty(&timeline)?;
        fs::write(timeline_path, timeline_str)?;
        
        // Create HEAD file pointing to main timeline
        fs::write(repo_path.join("HEAD"), "timeline: main\n")?;
        
        // Create empty pile
        let pile = Pile::new();
        
        // Create object store
        let object_store = ObjectStore::new(repo_path.join("objects"));
        
        Ok(Self {
            path: path.to_path_buf(),
            config,
            current_timeline: timeline,
            pile,
            object_store,
        })
    }
    
    /// Open an existing repository
    pub fn open(path: &Path) -> Result<Self> {
        // Find repository root by looking for .pocket directory
        let repo_root = Self::find_repository_root(path)?;
        let repo_path = repo_root.join(".pocket");
        
        if !repo_path.exists() {
            return Err(RepositoryError::NotARepository(path.to_path_buf()).into());
        }
        
        // Load configuration
        let config_path = repo_path.join("config.toml");
        let config_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        
        // Read HEAD to determine current timeline
        let head_content = fs::read_to_string(repo_path.join("HEAD"))?;
        let timeline_name = if head_content.starts_with("timeline: ") {
            head_content.trim_start_matches("timeline: ").trim()
        } else {
            return Err(anyhow!("Invalid HEAD format"));
        };
        
        // Load current timeline
        let timeline_path = repo_path.join("timelines").join(format!("{}.toml", timeline_name));
        let timeline_str = fs::read_to_string(timeline_path)?;
        let current_timeline: Timeline = toml::from_str(&timeline_str)?;
        
        // Load current pile
        let pile = Pile::load(&repo_path.join("piles").join("current.toml"))?;
        
        // Create object store
        let object_store = ObjectStore::new(repo_path.join("objects"));
        
        Ok(Self {
            path: repo_root,
            config,
            current_timeline,
            pile,
            object_store,
        })
    }
    
    /// Find the repository root by traversing up the directory tree
    fn find_repository_root(start_path: &Path) -> Result<PathBuf> {
        let mut current = start_path.to_path_buf();
        
        loop {
            if current.join(".pocket").exists() {
                return Ok(current);
            }
            
            if !current.pop() {
                return Err(RepositoryError::NotARepository(start_path.to_path_buf()).into());
            }
        }
    }
    
    /// Get the current status of the repository
    pub fn status(&self) -> Result<RepoStatus> {
        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut conflicts = Vec::new();

        // Get the current tree from HEAD
        let head_tree = if let Some(head) = &self.current_timeline.head {
            let shove = Shove::load(&self.path.join(".pocket").join("shoves").join(format!("{}.toml", head.as_str())))?;
            let tree_path = self.path.join(".pocket").join("objects").join(shove.root_tree_id.as_str());
            if tree_path.exists() {
                let tree_content = fs::read_to_string(&tree_path)?;
                Some(toml::from_str::<Tree>(&tree_content)?)
            } else {
                None
            }
        } else {
            None
        };

        // Walk the working directory
        let walker = walkdir::WalkDir::new(&self.path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.is_ignored(e.path()));

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();
            
            // Skip the .pocket directory
            if path.starts_with(self.path.join(".pocket")) {
                continue;
            }

            // Only process files
            if !entry.file_type().is_file() {
                continue;
            }

            let relative_path = path.strip_prefix(&self.path)?.to_path_buf();

            // Check if file is in current pile
            if self.pile.entries.contains_key(&relative_path) {
                continue;
            }

            // Check if file is tracked (exists in HEAD tree)
            if let Some(ref head_tree) = head_tree {
                let entry_path = relative_path.to_string_lossy().to_string();
                if let Some(head_entry) = head_tree.entries.iter().find(|e| e.name == entry_path) {
                    // File is tracked, check if modified
                    let current_content = fs::read(path)?;
                    let current_id = ObjectId::from_content(&current_content);
                    
                    if current_id != head_entry.id {
                        modified_files.push(relative_path);
                    }
                } else {
                    // File is not in HEAD tree
                    untracked_files.push(relative_path);
                }
            } else {
                // No HEAD tree, all files are untracked
                untracked_files.push(relative_path);
            }
        }

        // Check for conflicts
        let conflicts_dir = self.path.join(".pocket").join("conflicts");
        if conflicts_dir.exists() {
            for entry in fs::read_dir(conflicts_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    conflicts.push(PathBuf::from(entry.file_name()));
                }
            }
        }

        Ok(RepoStatus {
            current_timeline: self.current_timeline.name.clone(),
            head_shove: self.current_timeline.head.clone(),
            piled_files: self.pile.entries.values().cloned().collect(),
            modified_files,
            untracked_files,
            conflicts,
        })
    }
    
    /// Check if a path should be ignored
    fn is_ignored(&self, path: &Path) -> bool {
        // Always ignore .pocket directory
        if path.starts_with(self.path.join(".pocket")) {
            return true;
        }

        // Check if .pocketignore file exists and read patterns from it
        let ignore_path = self.path.join(".pocketignore");
        let ignore_patterns = if ignore_path.exists() {
            match std::fs::read_to_string(&ignore_path) {
                Ok(content) => {
                    content.lines()
                        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                        .map(|line| line.trim().to_string())
                        .collect::<Vec<String>>()
                },
                Err(_) => self.config.core.ignore_patterns.clone(),
            }
        } else {
            self.config.core.ignore_patterns.clone()
        };

        // Check against ignore patterns
        for pattern in &ignore_patterns {
            if let Ok(matcher) = glob::Pattern::new(pattern) {
                if let Ok(relative_path) = path.strip_prefix(&self.path) {
                    if matcher.matches_path(relative_path) {
                        return true;
                    }
                }
            }
        }

        false
    }
    
    /// Create a new shove (commit) from the current pile
    pub fn create_shove(&mut self, message: &str) -> Result<ShoveId> {
        // Create a tree from the pile
        let tree = self.create_tree_from_pile()?;
        let tree_id = self.object_store.store_tree(&tree)?;
        
        // Get parent shove(s)
        let parent_ids = if let Some(head) = &self.current_timeline.head {
            vec![head.clone()]
        } else {
            vec![]
        };
        
        // Create the shove
        let author = Author {
            name: self.config.user.name.clone(),
            email: self.config.user.email.clone(),
            timestamp: Utc::now(),
        };
        
        let shove = Shove::new(&self.pile, parent_ids, author, message, tree_id);
        let shove_id = shove.id.clone();
        
        // Save the shove
        let shove_path = self.path.join(".pocket").join("shoves").join(format!("{}.toml", shove.id.as_str()));
        shove.save(&shove_path)?;
        
        // Update the current timeline
        self.current_timeline.update_head(shove_id.clone());
        let timeline_path = self.path.join(".pocket").join("timelines").join(format!("{}.toml", self.current_timeline.name));
        self.current_timeline.save(&timeline_path)?;
        
        Ok(shove_id)
    }
    
    /// Create a tree object from the current pile
    fn create_tree_from_pile(&self) -> Result<Tree> {
        let mut entries = Vec::new();
        
        for (path, entry) in &self.pile.entries {
            let name = path.file_name()
                .ok_or_else(|| anyhow!("Invalid path: {}", path.display()))?
                .to_string_lossy()
                .into_owned();
                
            let tree_entry = TreeEntry {
                name,
                id: entry.object_id.clone(),
                entry_type: EntryType::File,
                permissions: 0o644, // Default file permissions
            };
            
            entries.push(tree_entry);
        }
        
        Ok(Tree { entries })
    }
    
    // Additional methods would be implemented here:
    // - pile_file: Add file to pile
    // - unpile_file: Remove file from pile
    // - create_shove: Create a new shove (commit)
    // - switch_timeline: Switch to a different timeline
    // - create_timeline: Create a new timeline
    // - merge_timeline: Merge another timeline into current
    // - etc.
} 