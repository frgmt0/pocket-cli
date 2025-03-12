//! Tests for the backup plugin
//!
//! These tests verify that the backup plugin works correctly.

use std::path::Path;
use std::fs;
use assert_fs::prelude::*;
use predicates::prelude::*;

use pocket_cli::plugins::backup::BackupPlugin;
use pocket_cli::plugins::{Plugin, PluginConfig};

// Import common test utilities
use crate::common;

#[test]
fn test_backup_plugin_creation() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let data_dir = temp.path();
    
    // Create a backup plugin
    let plugin = BackupPlugin::new(data_dir);
    
    // Verify that the plugin has the correct name and version
    assert_eq!(plugin.name(), "backup", "Plugin name should be 'backup'");
    assert!(plugin.version().len() > 0, "Plugin version should not be empty");
    assert!(plugin.description().len() > 0, "Plugin description should not be empty");
}

#[test]
fn test_backup_creation_and_listing() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let data_dir = temp.path();
    
    // Create some test data
    let snippets_dir = data_dir.join("snippets");
    fs::create_dir_all(&snippets_dir).expect("Failed to create snippets directory");
    
    let snippet1_path = snippets_dir.join("snippet1.json");
    fs::write(&snippet1_path, r#"{"name":"snippet1","content":"test content"}"#)
        .expect("Failed to write snippet1");
    
    let snippet2_path = snippets_dir.join("snippet2.json");
    fs::write(&snippet2_path, r#"{"name":"snippet2","content":"more test content"}"#)
        .expect("Failed to write snippet2");
    
    // Create a backup plugin
    let mut plugin = BackupPlugin::new(data_dir);
    
    // Initialize the plugin with a test configuration
    let mut config = PluginConfig {
        name: "backup".to_string(),
        enabled: true,
        options: std::collections::HashMap::new(),
    };
    
    // Set the backup directory to a subdirectory of the temp directory
    let backup_dir = temp.path().join("backups");
    let backup_config = serde_json::json!({
        "backup_dir": backup_dir,
        "max_backups": 3,
        "auto_backup": true,
        "backup_frequency": 1,
        "last_backup": null
    });
    config.options.insert("backup".to_string(), backup_config);
    
    plugin.initialize(&config).expect("Failed to initialize plugin");
    
    // Create a backup
    let description = "Test backup";
    let args = vec![description.to_string()];
    plugin.execute("backup", &args).expect("Failed to create backup");
    
    // List backups
    let result = plugin.execute("list", &[]).expect("Failed to list backups");
    
    // Verify that the backup directory exists
    assert!(backup_dir.exists(), "Backup directory should exist");
    
    // Verify that there is at least one backup
    let backups = fs::read_dir(&backup_dir).expect("Failed to read backup directory");
    let backup_count = backups.count();
    assert!(backup_count > 0, "Should have at least one backup");
}

#[test]
fn test_backup_restore() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let data_dir = temp.path();
    
    // Create some initial test data
    let snippets_dir = data_dir.join("snippets");
    fs::create_dir_all(&snippets_dir).expect("Failed to create snippets directory");
    
    let snippet1_path = snippets_dir.join("snippet1.json");
    fs::write(&snippet1_path, r#"{"name":"snippet1","content":"initial content"}"#)
        .expect("Failed to write snippet1");
    
    // Create a backup plugin
    let mut plugin = BackupPlugin::new(data_dir);
    
    // Initialize the plugin with a test configuration
    let mut config = PluginConfig {
        name: "backup".to_string(),
        enabled: true,
        options: std::collections::HashMap::new(),
    };
    
    // Set the backup directory to a subdirectory of the temp directory
    let backup_dir = temp.path().join("backups");
    let backup_config = serde_json::json!({
        "backup_dir": backup_dir,
        "max_backups": 3,
        "auto_backup": true,
        "backup_frequency": 1,
        "last_backup": null
    });
    config.options.insert("backup".to_string(), backup_config);
    
    plugin.initialize(&config).expect("Failed to initialize plugin");
    
    // Create a backup
    let description = "Initial backup";
    let args = vec![description.to_string()];
    plugin.execute("backup", &args).expect("Failed to create backup");
    
    // List backups to get the backup ID
    let backups = plugin.list_backups().expect("Failed to list backups");
    assert!(!backups.is_empty(), "Should have at least one backup");
    let backup_id = backups[0].id.clone();
    
    // Modify the data
    fs::write(&snippet1_path, r#"{"name":"snippet1","content":"modified content"}"#)
        .expect("Failed to modify snippet1");
    
    // Add a new snippet
    let snippet2_path = snippets_dir.join("snippet2.json");
    fs::write(&snippet2_path, r#"{"name":"snippet2","content":"new content"}"#)
        .expect("Failed to write snippet2");
    
    // Restore the backup
    let restore_args = vec![backup_id.clone()];
    plugin.execute("restore", &restore_args).expect("Failed to restore backup");
    
    // Verify that the data was restored to its initial state
    let restored_content = fs::read_to_string(&snippet1_path).expect("Failed to read restored snippet1");
    assert_eq!(restored_content, r#"{"name":"snippet1","content":"initial content"}"#, "Snippet1 should be restored to initial content");
    
    // Verify that snippet2 doesn't exist anymore (since it wasn't in the backup)
    assert!(!snippet2_path.exists(), "Snippet2 should not exist after restore");
}

#[test]
fn test_backup_pruning() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let data_dir = temp.path();
    
    // Create a backup plugin
    let mut plugin = BackupPlugin::new(data_dir);
    
    // Initialize the plugin with a test configuration
    let mut config = PluginConfig {
        name: "backup".to_string(),
        enabled: true,
        options: std::collections::HashMap::new(),
    };
    
    // Set the backup directory to a subdirectory of the temp directory
    let backup_dir = temp.path().join("backups");
    let backup_config = serde_json::json!({
        "backup_dir": backup_dir,
        "max_backups": 2, // Only keep 2 backups
        "auto_backup": true,
        "backup_frequency": 1,
        "last_backup": null
    });
    config.options.insert("backup".to_string(), backup_config);
    
    plugin.initialize(&config).expect("Failed to initialize plugin");
    
    // Create 3 backups
    for i in 1..=3 {
        let description = format!("Backup {}", i);
        let args = vec![description];
        plugin.execute("backup", &args).expect("Failed to create backup");
        
        // Add a small delay to ensure different timestamps
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    
    // List backups
    let backups = plugin.list_backups().expect("Failed to list backups");
    
    // Verify that only 2 backups remain (the newest ones)
    assert_eq!(backups.len(), 2, "Should have exactly 2 backups after pruning");
    
    // Verify that the remaining backups are the newest ones
    assert_eq!(backups[0].description, "Backup 3", "First backup should be 'Backup 3'");
    assert_eq!(backups[1].description, "Backup 2", "Second backup should be 'Backup 2'");
} 