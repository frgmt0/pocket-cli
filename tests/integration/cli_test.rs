//! Integration tests for the CLI commands
//! 
//! These tests verify that the CLI commands work correctly by executing
//! them and checking their output.

use std::process::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Sets up a temporary directory for testing CLI commands
/// 
/// Returns a TempDir instance that will be automatically cleaned up when dropped
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

#[test]
/// Test the 'new-repo' command
/// 
/// This test verifies that:
/// 1. The 'new-repo' command creates a new repository
/// 2. The repository contains the expected structure
fn test_new_repo_command() {
    let temp_dir = setup_test_dir();
    let repo_path = temp_dir.path();
    
    // Run the new-repo command
    let output = Command::new("cargo")
        .args(&["run", "--", "new-repo"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Verify .pocket directory exists
    let pocket_dir = repo_path.join(".pocket");
    assert!(pocket_dir.exists(), ".pocket directory was not created");
    
    // Verify config file exists
    let config_file = pocket_dir.join("config.toml");
    assert!(config_file.exists(), "config.toml was not created");
}

#[test]
/// Test the 'add' and 'list' commands for snippets
/// 
/// This test verifies that:
/// 1. A snippet can be added using the 'add' command
/// 2. The snippet can be listed using the 'list' command
fn test_add_and_list_snippets() {
    let temp_dir = setup_test_dir();
    let test_dir = temp_dir.path();
    
    // Create a test snippet file
    let snippet_file = test_dir.join("test_snippet.rs");
    fs::write(&snippet_file, "fn main() { println!(\"Hello, world!\"); }").expect("Failed to write snippet file");
    
    // Run the add command
    let output = Command::new("cargo")
        .args(&["run", "--", "add", "-f", snippet_file.to_str().unwrap(), "-t", "Test Snippet", "-l", "rust"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Add command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Run the list command
    let output = Command::new("cargo")
        .args(&["run", "--", "list"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "List command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("Test Snippet"), "List output does not contain the added snippet");
}

#[test]
/// Test the 'pile' and 'status' commands for VCS
/// 
/// This test verifies that:
/// 1. A repository can be initialized
/// 2. Files can be created and added to the pile
/// 3. The status command correctly shows piled files
fn test_pile_and_status_commands() {
    let temp_dir = setup_test_dir();
    let repo_path = temp_dir.path();
    
    // Initialize a repository
    let output = Command::new("cargo")
        .args(&["run", "--", "new-repo"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "new-repo command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Create a test file
    let test_file = repo_path.join("test.txt");
    fs::write(&test_file, "Hello, world!").expect("Failed to write test file");
    
    // Run the pile command
    let output = Command::new("cargo")
        .args(&["run", "--", "pile", test_file.to_str().unwrap()])
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "pile command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    // Run the status command
    let output = Command::new("cargo")
        .args(&["run", "--", "status"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "status command failed: {:?}", String::from_utf8_lossy(&output.stderr));
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("test.txt"), "Status output does not show the piled file");
} 