//! Unit tests for the ignore functionality
//! 
//! These tests verify that the .pocketignore file works correctly.

mod common;
use common::{create_temp_dir, setup_test_repository, create_test_file};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket::vcs::Repository;
    use std::fs;
    
    #[test]
    /// Test that ignored files are not added to the repository
    fn test_ignore_files() {
        let temp_dir = create_temp_dir();
        let repo_path = setup_test_repository(temp_dir.path());
        
        // Create a .pocketignore file with patterns
        let ignore_content = "*.log\ntmp/\n";
        fs::write(repo_path.join(".pocketignore"), ignore_content)
            .expect("Failed to write .pocketignore file");
        
        // Create test files
        create_test_file(repo_path.join("test.txt"), "Regular file");
        create_test_file(repo_path.join("test.log"), "Log file that should be ignored");
        fs::create_dir_all(repo_path.join("tmp")).expect("Failed to create tmp directory");
        create_test_file(repo_path.join("tmp/temp.txt"), "Temp file that should be ignored");
        
        // Open the repository
        let mut repo = Repository::open(&repo_path).expect("Failed to open repository");
        
        // Add all files (should respect ignore patterns)
        repo.add_all().expect("Failed to add all files");
        
        // Get staged files
        let staged_files = repo.get_staged_files().expect("Failed to get staged files");
        
        // Verify that test.txt is staged
        assert!(staged_files.iter().any(|p| p.ends_with("test.txt")), "test.txt should be staged");
        
        // Verify that test.log is not staged (ignored)
        assert!(!staged_files.iter().any(|p| p.ends_with("test.log")), "test.log should be ignored");
        
        // Verify that tmp/temp.txt is not staged (ignored)
        assert!(!staged_files.iter().any(|p| p.ends_with("temp.txt")), "tmp/temp.txt should be ignored");
    }
    
    #[test]
    /// Test adding and removing ignore patterns
    fn test_add_remove_ignore_patterns() {
        let temp_dir = create_temp_dir();
        let repo_path = setup_test_repository(temp_dir.path());
        
        // Open the repository
        let mut repo = Repository::open(&repo_path).expect("Failed to open repository");
        
        // Add a new ignore pattern
        repo.add_ignore_pattern("*.new_pattern").expect("Failed to add ignore pattern");
        
        // Verify the new pattern is in the repository
        let ignore_patterns = repo.get_ignore_patterns().expect("Failed to get ignore patterns");
        assert!(ignore_patterns.contains("*.new_pattern"), "New pattern was not added");
        
        // Remove the new pattern
        repo.remove_ignore_pattern("*.new_pattern").expect("Failed to remove ignore pattern");
        
        // Verify the new pattern is removed
        let updated_ignore_patterns = repo.get_ignore_patterns().expect("Failed to get updated ignore patterns");
        assert!(!updated_ignore_patterns.contains("*.new_pattern"), "New pattern was not removed");
    }
} 