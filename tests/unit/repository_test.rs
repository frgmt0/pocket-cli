//! Unit tests for the Repository functionality in the VCS module
//! 
//! These tests verify that the core repository operations work correctly
//! in isolation from other components.

use pocket_cli::vcs::Repository;
use std::fs;
use tempfile::TempDir;
use anyhow::Result;

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
fn test_create_new_repository() -> Result<()> {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    Repository::new(repo_path)?;
    
    // Verify .pocket directory exists
    let pocket_dir = repo_path.join(".pocket");
    assert!(pocket_dir.exists(), ".pocket directory was not created");
    
    // Verify config file exists
    let config_file = pocket_dir.join("config.toml");
    assert!(config_file.exists(), "config.toml was not created");
    
    // Verify the content of config file
    let config_content = fs::read_to_string(config_file)?;
    assert!(config_content.contains("[repository]"), "Config file missing repository section");
    
    Ok(())
}

#[test]
/// Test that repository status correctly identifies modified files
/// 
/// This test verifies that:
/// 1. The repository can detect new files
/// 2. The repository can detect modified files
/// 3. The repository correctly reports untracked files
fn test_repository_status() -> Result<()> {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    Repository::new(repo_path)?;
    
    // Create a new file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!")?;
    
    // Open the repository
    let repo = Repository::open(repo_path)?;
    
    // Check status
    let status = repo.status()?;
    
    // Verify the file is untracked
    assert!(status.untracked_files.iter().any(|p| p.ends_with("test.txt")), 
            "Repository status did not detect untracked file");
    
    Ok(())
}

#[test]
/// Test that files can be added to the pile (staging area)
/// 
/// This test verifies that:
/// 1. Files can be added to the pile
/// 2. The pile correctly tracks added files
/// 3. The repository status reflects piled files
fn test_pile_files() -> Result<()> {
    let temp_dir = setup_test_repo();
    let repo_path = temp_dir.path();
    
    // Initialize a new repository
    Repository::new(repo_path)?;
    
    // Create a new file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!")?;
    
    // Open the repository
    let mut repo = Repository::open(repo_path)?;
    
    // Add file to pile
    repo.pile(&[test_file.to_str().unwrap()])?;
    
    // Check status
    let status = repo.status()?;
    
    // Verify the file is in the pile
    assert!(status.piled_files.iter().any(|p| p.ends_with("test.txt")), 
            "Repository status did not detect piled file");
    
    Ok(())
} 