//! Unit tests for the Snippet storage functionality
//! 
//! These tests verify that snippets can be properly stored and retrieved.

use std::path::Path;

mod common;
use common::{create_temp_dir, create_test_file};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket::models::Snippet;
    use pocket::storage::SnippetStorage;
    use uuid::Uuid;
    
    #[test]
    /// Test that a snippet can be saved and retrieved
    fn test_save_and_retrieve_snippet() {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path).expect("Failed to create snippet storage");
        
        // Create a test snippet
        let snippet = Snippet {
            id: Uuid::new_v4(),
            title: "Test Snippet".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["test".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        };
        
        // Save the snippet
        let result = storage.save_snippet(&snippet);
        assert!(result.is_ok(), "Failed to save snippet: {:?}", result.err());
        
        // Retrieve the snippet
        let retrieved = storage.get_snippet(&snippet.id);
        assert!(retrieved.is_ok(), "Failed to retrieve snippet: {:?}", retrieved.err());
        
        let retrieved_snippet = retrieved.unwrap();
        assert_eq!(retrieved_snippet.id, snippet.id, "Snippet ID does not match");
        assert_eq!(retrieved_snippet.title, snippet.title, "Snippet title does not match");
        assert_eq!(retrieved_snippet.content, snippet.content, "Snippet content does not match");
    }
    
    #[test]
    /// Test that snippets can be searched by title
    fn test_search_snippets_by_title() {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path).expect("Failed to create snippet storage");
        
        // Create test snippets
        let snippet1 = Snippet {
            id: Uuid::new_v4(),
            title: "Rust Hello World".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        };
        
        let snippet2 = Snippet {
            id: Uuid::new_v4(),
            title: "Python Hello World".to_string(),
            content: "print(\"Hello, world!\")".to_string(),
            language: Some("python".to_string()),
            tags: vec!["python".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        };
        
        // Save the snippets
        storage.save_snippet(&snippet1).expect("Failed to save snippet1");
        storage.save_snippet(&snippet2).expect("Failed to save snippet2");
        
        // Search for snippets containing "Rust"
        let results = storage.search_snippets("Rust", None);
        assert_eq!(results.len(), 1, "Expected 1 result for 'Rust' search");
        assert_eq!(results[0].id, snippet1.id, "Wrong snippet returned for 'Rust' search");
        
        // Search for snippets containing "Hello"
        let results = storage.search_snippets("Hello", None);
        assert_eq!(results.len(), 2, "Expected 2 results for 'Hello' search");
    }
    
    #[test]
    /// Test that snippets can be deleted
    fn test_delete_snippet() {
        let temp_dir = create_temp_dir();
        let storage_path = temp_dir.path().join("snippets");
        
        // Create a snippet storage
        let mut storage = SnippetStorage::new(&storage_path).expect("Failed to create snippet storage");
        
        // Create a test snippet
        let snippet = Snippet {
            id: Uuid::new_v4(),
            title: "Test Snippet".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["test".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        };
        
        // Save the snippet
        storage.save_snippet(&snippet).expect("Failed to save snippet");
        
        // Delete the snippet
        let result = storage.delete_snippet(&snippet.id);
        assert!(result.is_ok(), "Failed to delete snippet: {:?}", result.err());
        
        // Try to retrieve the deleted snippet
        let retrieved = storage.get_snippet(&snippet.id);
        assert!(retrieved.is_err(), "Deleted snippet should not be retrievable");
    }
} 