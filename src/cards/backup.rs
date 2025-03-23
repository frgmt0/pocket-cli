//! Backup card for Pocket CLI
//!
//! This card provides functionality for backing up and restoring snippets and repositories.

use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

use crate::cards::{Card, CardConfig, CardCommand};

/// Configuration for the backup card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCardConfig {
    /// Directory where backups are stored
    pub backup_dir: PathBuf,
    
    /// Maximum number of backups to keep
    pub max_backups: usize,
    
    /// Whether to automatically backup on exit
    pub auto_backup: bool,
    
    /// Backup frequency in days (0 means no automatic backups)
    pub backup_frequency: u32,
    
    /// Date of the last backup
    pub last_backup: Option<DateTime<Utc>>,
}

impl Default for BackupCardConfig {
    fn default() -> Self {
        Self {
            backup_dir: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("pocket")
                .join("backups"),
            max_backups: 5,
            auto_backup: true,
            backup_frequency: 1,
            last_backup: None,
        }
    }
}

/// Metadata for a backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Unique identifier for the backup
    pub id: String,
    
    /// Date and time when the backup was created
    pub created_at: DateTime<Utc>,
    
    /// Description of the backup
    pub description: String,
    
    /// Number of snippets in the backup
    pub snippet_count: usize,
    
    /// Number of repositories in the backup
    pub repository_count: usize,
    
    /// Size of the backup in bytes
    pub size: u64,
}

/// Card for backing up and restoring snippets and repositories
pub struct BackupCard {
    /// Name of the card
    name: String,
    
    /// Version of the card
    version: String,
    
    /// Description of the card
    description: String,
    
    /// Configuration for the card
    config: BackupCardConfig,
    
    /// Path to the Pocket data directory (kept for future use)
    _data_dir: PathBuf,
}

impl BackupCard {
    /// Creates a new backup card
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            name: "backup".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Provides functionality for backing up and restoring snippets and repositories".to_string(),
            config: BackupCardConfig::default(),
            _data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Creates a backup of the current state
    pub fn create_backup(&self, description: &str) -> Result<BackupMetadata> {
        // Ensure the backup directory exists
        fs::create_dir_all(&self.config.backup_dir)
            .context("Failed to create backup directory")?;
        
        // Generate a unique ID for the backup
        let backup_id = format!("backup_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let backup_dir = self.config.backup_dir.join(&backup_id);
        
        // Create the backup directory
        fs::create_dir(&backup_dir)
            .context("Failed to create backup directory")?;
        
        // Copy the data directory to the backup directory
        self.copy_directory(&self._data_dir, &backup_dir)
            .context("Failed to copy data directory")?;
        
        // Count snippets and repositories
        let snippet_count = self.count_snippets(&backup_dir)?;
        let repository_count = self.count_repositories(&backup_dir)?;
        
        // Calculate the size of the backup
        let size = self.directory_size(&backup_dir)?;
        
        // Create metadata
        let metadata = BackupMetadata {
            id: backup_id,
            created_at: Utc::now(),
            description: description.to_string(),
            snippet_count,
            repository_count,
            size,
        };
        
        // Save metadata
        let metadata_path = backup_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, metadata_json)
            .context("Failed to write backup metadata")?;
        
        // Prune old backups if necessary
        self.prune_old_backups()?;
        
        Ok(metadata)
    }
    
    /// Restores a backup
    pub fn restore_backup(&self, backup_id: &str) -> Result<()> {
        let backup_dir = self.config.backup_dir.join(backup_id);
        
        // Check if the backup exists
        if !backup_dir.exists() {
            anyhow::bail!("Backup '{}' not found", backup_id);
        }
        
        // Read metadata to verify it's a valid backup
        let metadata_path = backup_dir.join("metadata.json");
        if !metadata_path.exists() {
            anyhow::bail!("Invalid backup: metadata.json not found");
        }
        
        // Create a backup of the current state before restoring
        let current_backup_id = format!("pre_restore_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let current_backup_dir = self.config.backup_dir.join(&current_backup_id);
        
        // Create the backup directory
        fs::create_dir(&current_backup_dir)
            .context("Failed to create backup directory for current state")?;
        
        // Copy the current data directory to the backup directory
        self.copy_directory(&self._data_dir, &current_backup_dir)
            .context("Failed to backup current state")?;
        
        // Clear the current data directory
        self.clear_directory(&self._data_dir)
            .context("Failed to clear data directory")?;
        
        // Copy the backup to the data directory
        self.copy_directory(&backup_dir, &self._data_dir)
            .context("Failed to restore backup")?;
        
        Ok(())
    }
    
    /// Lists all available backups
    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        // Ensure the backup directory exists
        if !self.config.backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups = Vec::new();
        
        // Iterate through all entries in the backup directory
        for entry in fs::read_dir(&self.config.backup_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it's a directory
            if path.is_dir() {
                // Check if it contains a metadata.json file
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    // Read and parse the metadata
                    let metadata_json = fs::read_to_string(&metadata_path)?;
                    let metadata: BackupMetadata = serde_json::from_str(&metadata_json)?;
                    backups.push(metadata);
                }
            }
        }
        
        // Sort backups by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    /// Deletes a backup
    pub fn delete_backup(&self, backup_id: &str) -> Result<()> {
        let backup_dir = self.config.backup_dir.join(backup_id);
        
        // Check if the backup exists
        if !backup_dir.exists() {
            anyhow::bail!("Backup '{}' not found", backup_id);
        }
        
        // Delete the backup directory
        fs::remove_dir_all(&backup_dir)
            .context("Failed to delete backup")?;
        
        Ok(())
    }
    
    /// Prunes old backups to stay within the maximum limit
    fn prune_old_backups(&self) -> Result<()> {
        // List all backups
        let mut backups = self.list_backups()?;
        
        // If we're within the limit, do nothing
        if backups.len() <= self.config.max_backups {
            return Ok(());
        }
        
        // Sort backups by creation date (oldest first)
        backups.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        
        // Delete the oldest backups until we're within the limit
        for backup in backups.iter().take(backups.len() - self.config.max_backups) {
            self.delete_backup(&backup.id)?;
        }
        
        Ok(())
    }
    
    /// Copies a directory recursively
    fn copy_directory(&self, src: &Path, dst: &Path) -> Result<()> {
        // Create the destination directory if it doesn't exist
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        
        // Iterate through all entries in the source directory
        for entry in walkdir::WalkDir::new(src) {
            let entry = entry?;
            let src_path = entry.path();
            let rel_path = src_path.strip_prefix(src)?;
            let dst_path = dst.join(rel_path);
            
            if src_path.is_dir() {
                // Create the directory in the destination
                fs::create_dir_all(&dst_path)?;
            } else {
                // Copy the file
                fs::copy(src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Clears a directory without deleting the directory itself
    fn clear_directory(&self, dir: &Path) -> Result<()> {
        // Check if the directory exists
        if !dir.exists() {
            return Ok(());
        }
        
        // Iterate through all entries in the directory
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Recursively delete the directory
                fs::remove_dir_all(&path)?;
            } else {
                // Delete the file
                fs::remove_file(&path)?;
            }
        }
        
        Ok(())
    }
    
    /// Counts the number of snippets in a directory
    fn count_snippets(&self, dir: &Path) -> Result<usize> {
        let snippets_dir = dir.join("snippets");
        
        if !snippets_dir.exists() {
            return Ok(0);
        }
        
        let count = walkdir::WalkDir::new(&snippets_dir)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "json"))
            .count();
        
        Ok(count)
    }
    
    /// Counts the number of repositories in a directory
    fn count_repositories(&self, dir: &Path) -> Result<usize> {
        let repos_dir = dir.join("repositories");
        
        if !repos_dir.exists() {
            return Ok(0);
        }
        
        let count = walkdir::WalkDir::new(&repos_dir)
            .max_depth(1)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            .count();
        
        Ok(count)
    }
    
    /// Calculates the size of a directory in bytes
    fn directory_size(&self, dir: &Path) -> Result<u64> {
        let mut size = 0;
        
        for entry in walkdir::WalkDir::new(dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
        
        Ok(size)
    }
}

impl Card for BackupCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn _description(&self) -> &str {
        &self.description
    }
    
    fn _initialize(&mut self, config: &CardConfig) -> Result<()> {
        // If there are options in the card config, try to parse them
        if let Some(options_value) = config.options.get("backup") {
            if let Ok(options) = serde_json::from_value::<BackupCardConfig>(options_value.clone()) {
                self.config = options;
            }
        }
        
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "backup" => {
                let description = args.first().map(|s| s.as_str()).unwrap_or("Manual backup");
                let metadata = self.create_backup(description)?;
                println!("Backup created: {}", metadata.id);
                println!("Description: {}", metadata.description);
                println!("Created at: {}", metadata.created_at);
                println!("Snippets: {}", metadata.snippet_count);
                println!("Repositories: {}", metadata.repository_count);
                println!("Size: {} bytes", metadata.size);
                Ok(())
            },
            "restore" => {
                if args.is_empty() {
                    anyhow::bail!("Backup ID is required");
                }
                let backup_id = &args[0];
                self.restore_backup(backup_id)?;
                println!("Backup '{}' restored successfully", backup_id);
                Ok(())
            },
            "list" => {
                let backups = self.list_backups()?;
                if backups.is_empty() {
                    println!("No backups found");
                } else {
                    println!("Available backups:");
                    for backup in backups {
                        println!("ID: {}", backup.id);
                        println!("  Description: {}", backup.description);
                        println!("  Created at: {}", backup.created_at);
                        println!("  Snippets: {}", backup.snippet_count);
                        println!("  Repositories: {}", backup.repository_count);
                        println!("  Size: {} bytes", backup.size);
                        println!();
                    }
                }
                Ok(())
            },
            "delete" => {
                if args.is_empty() {
                    anyhow::bail!("Backup ID is required");
                }
                let backup_id = &args[0];
                self.delete_backup(backup_id)?;
                println!("Backup '{}' deleted successfully", backup_id);
                Ok(())
            },
            _ => anyhow::bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "backup".to_string(),
                description: "Creates a backup of the current state".to_string(),
                usage: "pocket backup [description]".to_string(),
            },
            CardCommand {
                name: "restore".to_string(),
                description: "Restores a backup".to_string(),
                usage: "pocket restore <backup-id>".to_string(),
            },
            CardCommand {
                name: "list".to_string(),
                description: "Lists all available backups".to_string(),
                usage: "pocket backup list".to_string(),
            },
            CardCommand {
                name: "delete".to_string(),
                description: "Deletes a backup".to_string(),
                usage: "pocket backup delete <backup-id>".to_string(),
            },
        ]
    }
    
    fn cleanup(&mut self) -> Result<()> {
        // Nothing to clean up
        Ok(())
    }
} 