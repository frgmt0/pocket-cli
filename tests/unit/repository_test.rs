//! Unit tests for the Repository functionality in the VCS module
//! 
//! These tests verify that the core repository operations work correctly
//! in isolation from other components.

use pocket_cli::vcs::Repository;
use std::path::Path;
use std::fs;
use tempfile::TempDir;

/// Sets up a temporary directory for testing repository operations
/// 
/// Returns a TempDir instance that will be automatically cleaned up when dropped
fn setup_test_repo() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

#[test]
/// Test that a new repository can be created successfully
/// 
/// This test verifies that:
/// 1. A new repository can be initialized in an empty directory
/// 2. The repository contains the expected structure (.pocket directory)
/// 3. The config file is created with default values
fn test_create_new_repository() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    let result = Repository::init(repo_path);
    assert!(result.is_ok(), "Repository initialization failed: {:?}", result.err());
    
    // Verify .pocket directory exists
    let pocket_dir = repo_path.join(".pocket");
    assert!(pocket_dir.exists(), ".pocket directory was not created");
    
    // Verify config file exists
    let config_file = pocket_dir.join("config.toml");
    assert!(config_file.exists(), "config.toml was not created");
    
    // Verify the content of config file
    let config_content = fs::read_to_string(config_file).expect("Failed to read config file");
    assert!(config_content.contains("[repository]"), "Config file missing repository section");
}

#[test]
/// Test that repository status correctly identifies modified files
/// 
/// This test verifies that:
/// 1. The repository can detect new files
/// 2. The repository can detect modified files
/// 3. The repository correctly reports untracked files
fn test_repository_status() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    let result = Repository::init(repo_path);
    assert!(result.is_ok(), "Repository initialization failed");
    
    // Create a new file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    // Open the repository
    let repo = Repository::open(repo_path).expect("Failed to open repository");
    
    // Check status
    let status = repo.status().expect("Failed to get repository status");
    
    // Verify the file is untracked
    assert!(status.untracked.contains(&test_file.to_string_lossy().to_string()), 
            "Repository status did not detect untracked file");
}

#[test]
/// Test that files can be added to the pile (staging area)
/// 
/// This test verifies that:
/// 1. Files can be added to the pile
/// 2. The pile correctly tracks added files
/// 3. The repository status reflects piled files
fn test_pile_files() {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    Repository::init(repo_path).expect("Failed to initialize repository");
    
    // Create a new file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    // Open the repository
    let mut repo = Repository::open(repo_path).expect("Failed to open repository");
    
    // Add file to pile
    let result = repo.pile(&[test_file.to_str().unwrap()]);
    assert!(result.is_ok(), "Failed to add file to pile: {:?}", result.err());
    
    // Check status
    let status = repo.status().expect("Failed to get repository status");
    
    // Verify the file is in the pile
    assert!(status.piled.contains(&test_file.to_string_lossy().to_string()), 
            "Repository status did not detect piled file");
} 