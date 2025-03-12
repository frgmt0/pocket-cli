//! Unit tests for the Repository functionality
//! 
//! These tests verify that the Repository struct and its methods
//! work correctly in isolation.

use std::path::Path;
use tempfile::TempDir;
use std::fs;
use anyhow::Result;

mod common;
use common::{create_temp_dir, setup_test_repository, create_test_file};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket_cli::vcs::Repository;

    /// Creates a temporary directory for testing
    fn create_temp_dir() -> TempDir {
        TempDir::new().expect("Failed to create temporary directory")
    }
    
    /// Creates a test file with specified content
    fn create_test_file<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, content: C) {
        fs::write(path, content).expect("Failed to write test file");
    }

    #[test]
    /// Test that a new repository can be created successfully
    fn test_new_repository_creation() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        Repository::init(repo_path)?;
        
        // Verify .pocket directory exists
        assert!(repo_path.join(".pocket").exists(), ".pocket directory was not created");
        
        // Verify config.toml exists
        assert!(repo_path.join(".pocket/config.toml").exists(), "config.toml was not created");
        
        Ok(())
    }
    
    #[test]
    /// Test that an existing repository can be opened
    fn test_open_existing_repository() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository first
        Repository::init(repo_path)?;
        
        // Open the existing repository
        let result = Repository::open(repo_path);
        assert!(result.is_ok(), "Failed to open repository: {:?}", result.err());
        
        Ok(())
    }
    
    #[test]
    /// Test that opening a non-existent repository fails
    fn test_open_nonexistent_repository() {
        let temp_dir = create_temp_dir();
        let non_repo_path = temp_dir.path().join("non_existent");
        
        // Try to open a non-existent repository
        let result = Repository::open(&non_repo_path);
        assert!(result.is_err(), "Opening non-existent repository should fail");
    }
    
    #[test]
    /// Test adding files to the repository
    fn test_add_files_to_repository() {
        let temp_dir = create_temp_dir();
        let repo_path = setup_test_repository(temp_dir.path());
        
        // Create a test file
        let test_file_path = repo_path.join("test.txt");
        create_test_file(&test_file_path, "Hello, world!");
        
        // Open the repository
        let mut repo = Repository::open(&repo_path).expect("Failed to open repository");
        
        // Add the file to the repository
        let result = repo.add_file(&test_file_path);
        assert!(result.is_ok(), "Failed to add file to repository: {:?}", result.err());
        
        // Verify the file is in the staging area
        let staged_files = repo.get_staged_files().expect("Failed to get staged files");
        assert!(staged_files.contains(&test_file_path), "File was not added to staging area");
    }
} 