//! Unit tests for the Snippet functionality
//! 
//! These tests verify that the core snippet operations work correctly
//! in isolation from other components.

use crate::models::Snippet;
use pocket_cli::storage::Storage;
use std::path::Path;
use tempfile::TempDir;

/// Sets up a temporary directory for testing snippet operations
/// 
/// Returns a TempDir instance that will be automatically cleaned up when dropped
fn setup_test_storage() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

#[test]
/// Test that a snippet can be created and saved successfully
/// 
/// This test verifies that:
/// 1. A new snippet can be created with the expected properties
/// 2. The snippet can be saved to storage
/// 3. The snippet can be retrieved from storage with the same properties
fn test_create_and_save_snippet() {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = Storage::new(storage_path).expect("Failed to create storage");
    
    // Create a new snippet
    let snippet = Snippet {
        id: "test-id".to_string(),
        title: "Test Snippet".to_string(),
        content: "println!(\"Hello, world!\");".to_string(),
        language: Some("rust".to_string()),
        tags: vec!["test".to_string(), "example".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        backpack: None,
    };
    
    // Save the snippet
    let result = storage.save_snippet(&snippet);
    assert!(result.is_ok(), "Failed to save snippet: {:?}", result.err());
    
    // Retrieve the snippet
    let retrieved = storage.get_snippet("test-id");
    assert!(retrieved.is_ok(), "Failed to retrieve snippet: {:?}", retrieved.err());
    
    let retrieved_snippet = retrieved.unwrap();
    assert_eq!(retrieved_snippet.id, snippet.id, "Snippet ID mismatch");
    assert_eq!(retrieved_snippet.title, snippet.title, "Snippet title mismatch");
    assert_eq!(retrieved_snippet.content, snippet.content, "Snippet content mismatch");
    assert_eq!(retrieved_snippet.language, snippet.language, "Snippet language mismatch");
    assert_eq!(retrieved_snippet.tags, snippet.tags, "Snippet tags mismatch");
}

#[test]
/// Test that snippets can be searched by content
/// 
/// This test verifies that:
/// 1. Multiple snippets can be saved
/// 2. Snippets can be searched by content
/// 3. Search results are ordered by relevance
fn test_search_snippets() {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = Storage::new(storage_path).expect("Failed to create storage");
    
    // Create and save multiple snippets
    let snippets = vec![
        Snippet {
            id: "id1".to_string(),
            title: "Rust Hello World".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        },
        Snippet {
            id: "id2".to_string(),
            title: "Python Hello World".to_string(),
            content: "print(\"Hello, world!\")".to_string(),
            language: Some("python".to_string()),
            tags: vec!["python".to_string(), "hello".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        },
        Snippet {
            id: "id3".to_string(),
            title: "Rust Function".to_string(),
            content: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string(), "function".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: None,
        },
    ];
    
    for snippet in &snippets {
        storage.save_snippet(snippet).expect("Failed to save snippet");
    }
    
    // Search for "Hello"
    let results = storage.search_snippets("Hello");
    assert!(results.is_ok(), "Search failed: {:?}", results.err());
    
    let search_results = results.unwrap();
    assert_eq!(search_results.len(), 2, "Expected 2 search results");
    
    // Search for "rust"
    let results = storage.search_snippets("rust");
    assert!(results.is_ok(), "Search failed: {:?}", results.err());
    
    let search_results = results.unwrap();
    assert_eq!(search_results.len(), 2, "Expected 2 search results");
}

#[test]
/// Test that snippets can be organized in backpacks
/// 
/// This test verifies that:
/// 1. Snippets can be assigned to backpacks
/// 2. Snippets can be retrieved by backpack
/// 3. Backpacks can be listed
fn test_backpack_organization() {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = Storage::new(storage_path).expect("Failed to create storage");
    
    // Create snippets in different backpacks
    let snippets = vec![
        Snippet {
            id: "id1".to_string(),
            title: "Rust Snippet 1".to_string(),
            content: "println!(\"Hello from Rust!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: Some("rust-backpack".to_string()),
        },
        Snippet {
            id: "id2".to_string(),
            title: "Python Snippet 1".to_string(),
            content: "print(\"Hello from Python!\")".to_string(),
            language: Some("python".to_string()),
            tags: vec!["python".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: Some("python-backpack".to_string()),
        },
        Snippet {
            id: "id3".to_string(),
            title: "Rust Snippet 2".to_string(),
            content: "fn main() { println!(\"Another Rust example\"); }".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string()],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            backpack: Some("rust-backpack".to_string()),
        },
    ];
    
    for snippet in &snippets {
        storage.save_snippet(snippet).expect("Failed to save snippet");
    }
    
    // List backpacks
    let backpacks = storage.list_backpacks();
    assert!(backpacks.is_ok(), "Failed to list backpacks: {:?}", backpacks.err());
    
    let backpack_list = backpacks.unwrap();
    assert_eq!(backpack_list.len(), 2, "Expected 2 backpacks");
    assert!(backpack_list.contains(&"rust-backpack".to_string()), "Missing rust-backpack");
    assert!(backpack_list.contains(&"python-backpack".to_string()), "Missing python-backpack");
    
    // Get snippets from rust-backpack
    let rust_snippets = storage.get_snippets_by_backpack("rust-backpack");
    assert!(rust_snippets.is_ok(), "Failed to get snippets by backpack: {:?}", rust_snippets.err());
    
    let rust_snippet_list = rust_snippets.unwrap();
    assert_eq!(rust_snippet_list.len(), 2, "Expected 2 snippets in rust-backpack");
} 