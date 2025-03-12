//! Unit tests for the ignore functionality
//! 
//! These tests verify that the .pocketignore file works correctly.

use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use pocket_cli::vcs::Repository;
    use crate::common::{create_temp_dir, create_test_file};
    use std::fs;
    
    #[test]
    /// Test that ignored files are not added to the repository
    fn test_ignore_files() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        Repository::init(repo_path)?;
        
        // Create a .pocketignore file with patterns
        let ignore_content = "*.log\ntmp/\n";
        fs::write(repo_path.join(".pocketignore"), ignore_content)?;
        
        // Create test files
        create_test_file(repo_path.join("test.txt"), "Regular file")?;
        create_test_file(repo_path.join("test.log"), "Log file that should be ignored")?;
        fs::create_dir_all(repo_path.join("tmp"))?;
        create_test_file(repo_path.join("tmp/temp.txt"), "Temp file that should be ignored")?;
        
        // Open the repository
        let mut repo = Repository::open(repo_path)?;
        
        // Add all files (should respect ignore patterns)
        repo.add_all()?;
        
        // Get staged files
        let staged_files = repo.get_staged_files()?;
        
        // Verify that test.txt is staged
        assert!(staged_files.iter().any(|p| p.ends_with("test.txt")), "test.txt should be staged");
        
        // Verify that test.log is not staged (ignored)
        assert!(!staged_files.iter().any(|p| p.ends_with("test.log")), "test.log should be ignored");
        
        // Verify that tmp/temp.txt is not staged (ignored)
        assert!(!staged_files.iter().any(|p| p.ends_with("temp.txt")), "tmp/temp.txt should be ignored");
        
        Ok(())
    }
    
    #[test]
    /// Test adding and removing ignore patterns
    fn test_add_remove_ignore_patterns() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        Repository::init(repo_path)?;
        
        // Open the repository
        let mut repo = Repository::open(repo_path)?;
        
        // Add ignore patterns
        repo.add_ignore_pattern("*.log")?;
        repo.add_ignore_pattern("tmp/")?;
        
        // Verify patterns are in .pocketignore
        let ignore_content = fs::read_to_string(repo_path.join(".pocketignore"))?;
        assert!(ignore_content.contains("*.log"), ".pocketignore should contain *.log pattern");
        assert!(ignore_content.contains("tmp/"), ".pocketignore should contain tmp/ pattern");
        
        // Remove an ignore pattern
        repo.remove_ignore_pattern("*.log")?;
        
        // Verify pattern was removed
        let ignore_content = fs::read_to_string(repo_path.join(".pocketignore"))?;
        assert!(!ignore_content.contains("*.log"), ".pocketignore should not contain *.log pattern");
        assert!(ignore_content.contains("tmp/"), ".pocketignore should still contain tmp/ pattern");
        
        Ok(())
    }
} 