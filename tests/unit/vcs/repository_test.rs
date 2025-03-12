//! Unit tests for the Repository functionality
//! 
//! These tests verify that the Repository struct and its methods
//! work correctly in isolation.

use std::path::Path;

mod common;
use common::{create_temp_dir, setup_test_repository, create_test_file};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket::vcs::Repository;

    #[test]
    /// Test that a new repository can be created successfully
    fn test_new_repository_creation() {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        let result = Repository::init(repo_path);
        assert!(result.is_ok(), "Failed to create repository: {:?}", result.err());
        
        // Verify .pocket directory exists
        assert!(repo_path.join(".pocket").exists(), ".pocket directory was not created");
        
        // Verify config.toml exists
        assert!(repo_path.join(".pocket/config.toml").exists(), "config.toml was not created");
    }
    
    #[test]
    /// Test that an existing repository can be opened
    fn test_open_existing_repository() {
        let temp_dir = create_temp_dir();
        let repo_path = setup_test_repository(temp_dir.path());
        
        // Open the existing repository
        let result = Repository::open(&repo_path);
        assert!(result.is_ok(), "Failed to open repository: {:?}", result.err());
        
        let repo = result.unwrap();
        assert_eq!(repo.get_name(), "test-repo", "Repository name does not match");
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