//! Common testing utilities for Pocket tests
//! 
//! This module provides shared functionality for both unit and integration tests,
//! including test repository setup, temporary directories, and assertion helpers.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory for testing
/// 
/// This function creates a new temporary directory that will be automatically
/// deleted when the returned TempDir is dropped. Use this for tests that need
/// to create files or repositories.
/// 
/// # Returns
/// 
/// * `TempDir` - A temporary directory that will be cleaned up automatically
/// 
/// # Examples
/// 
/// ```
/// let temp_dir = create_temp_dir();
/// let path = temp_dir.path();
/// // Use path for testing...
/// // Directory will be automatically cleaned up when temp_dir goes out of scope
/// ```
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Sets up a test repository with initial structure
/// 
/// Creates a basic Pocket repository structure in the given directory,
/// including the .pocket directory and necessary configuration files.
/// 
/// # Arguments
/// 
/// * `dir_path` - Path where the test repository should be created
/// 
/// # Returns
/// 
/// * `PathBuf` - Path to the created repository
/// 
/// # Examples
/// 
/// ```
/// let temp_dir = create_temp_dir();
/// let repo_path = setup_test_repository(temp_dir.path());
/// // repo_path now points to a valid Pocket repository
/// ```
pub fn setup_test_repository(dir_path: &Path) -> PathBuf {
    let repo_path = dir_path.to_path_buf();
    
    // Create .pocket directory structure
    let pocket_dir = repo_path.join(".pocket");
    fs::create_dir_all(&pocket_dir).expect("Failed to create .pocket directory");
    
    // Create basic config.toml
    let config_content = r#"
[repository]
name = "test-repo"
created_at = "2023-01-01T00:00:00Z"
"#;
    
    fs::write(pocket_dir.join("config.toml"), config_content)
        .expect("Failed to write config.toml");
    
    // Create empty .pocketignore file
    fs::write(repo_path.join(".pocketignore"), "")
        .expect("Failed to create .pocketignore file");
    
    // Create directories for objects, timelines, etc.
    fs::create_dir_all(pocket_dir.join("objects")).expect("Failed to create objects directory");
    fs::create_dir_all(pocket_dir.join("timelines")).expect("Failed to create timelines directory");
    
    repo_path
}

/// Creates a test file with specified content
/// 
/// # Arguments
/// 
/// * `path` - Path where the file should be created
/// * `content` - Content to write to the file
/// 
/// # Examples
/// 
/// ```
/// let temp_dir = create_temp_dir();
/// create_test_file(temp_dir.path().join("test.txt"), "Hello, world!");
/// ```
pub fn create_test_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) {
    fs::write(path, content).expect("Failed to write test file");
}

/// Asserts that a file exists and optionally checks its content
/// 
/// # Arguments
/// 
/// * `path` - Path to the file to check
/// * `expected_content` - Optional content to verify
/// 
/// # Examples
/// 
/// ```
/// let temp_dir = create_temp_dir();
/// let file_path = temp_dir.path().join("test.txt");
/// create_test_file(&file_path, "Hello, world!");
/// assert_file_exists(&file_path, Some("Hello, world!"));
/// ```
pub fn assert_file_exists<P: AsRef<Path>>(path: P, expected_content: Option<&str>) {
    let path = path.as_ref();
    assert!(path.exists(), "File does not exist: {:?}", path);
    
    if let Some(content) = expected_content {
        let file_content = fs::read_to_string(path).expect("Failed to read file");
        assert_eq!(file_content, content, "File content does not match expected content");
    }
} 