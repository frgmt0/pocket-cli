//! Unit tests for the Snippet storage functionality
//! 
//! These tests verify that snippets can be properly stored and retrieved.

use anyhow::Result;

mod common;
use common::{create_temp_dir, create_test_file};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket_cli::models::Snippet;
    use pocket_cli::storage::SnippetStorage;
    use crate::common::{create_temp_dir, create_test_snippet};
    use uuid::Uuid;
    
    #[test]
    /// Test that a snippet can be saved and retrieved
    fn test_save_and_retrieve_snippet() -> Result<()> {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path)?;
        
        // Create a test snippet
        let snippet = create_test_snippet(
            "Test Snippet", 
            "println!(\"Hello, world!\");", 
            Some("rust")
        );
        
        // Save the snippet
        storage.save_snippet(&snippet)?;
        
        // Retrieve the snippet
        let retrieved = storage.get_snippet(&snippet.id)?;
        
        assert_eq!(retrieved.id, snippet.id, "Snippet ID does not match");
        assert_eq!(retrieved.title, snippet.title, "Snippet title does not match");
        assert_eq!(retrieved.content, snippet.content, "Snippet content does not match");
        
        Ok(())
    }
    
    #[test]
    /// Test that snippets can be searched by title
    fn test_search_snippets_by_title() -> Result<()> {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path)?;
        
        // Create test snippets
        let snippet1 = create_test_snippet(
            "Rust Hello World", 
            "println!(\"Hello, world!\");", 
            Some("rust")
        );
        
        let snippet2 = create_test_snippet(
            "Python Hello World", 
            "print(\"Hello, world!\")", 
            Some("python")
        );
        
        // Save the snippets
        storage.save_snippet(&snippet1)?;
        storage.save_snippet(&snippet2)?;
        
        // Search for snippets containing "Rust"
        let results = storage.search_snippets("Rust", None)?;
        assert_eq!(results.len(), 1, "Expected 1 result for 'Rust' search");
        assert_eq!(results[0].id, snippet1.id, "Wrong snippet returned for 'Rust' search");
        
        // Search for snippets containing "Hello"
        let results = storage.search_snippets("Hello", None)?;
        assert_eq!(results.len(), 2, "Expected 2 results for 'Hello' search");
        
        Ok(())
    }
    
    #[test]
    /// Test that snippets can be deleted
    fn test_delete_snippet() -> Result<()> {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path)?;
        
        // Create a test snippet
        let snippet = create_test_snippet(
            "Test Snippet", 
            "println!(\"Hello, world!\");", 
            Some("rust")
        );
        
        // Save the snippet
        storage.save_snippet(&snippet)?;
        
        // Delete the snippet
        let result = storage.delete_snippet(&snippet.id)?;
        assert!(result.is_ok(), "Failed to delete snippet: {:?}", result.err());
        
        // Try to retrieve the deleted snippet
        let retrieved = storage.get_snippet(&snippet.id)?;
        assert!(retrieved.is_err(), "Deleted snippet should not be retrievable");
        
        Ok(())
    }
} 