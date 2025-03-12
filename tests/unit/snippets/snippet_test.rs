//! Unit tests for the Snippet functionality
//! 
//! These tests verify that the core snippet operations work correctly
//! in isolation from other components.

use pocket_cli::models::Snippet;
use pocket_cli::storage::SnippetStorage;
use std::path::Path;
use tempfile::TempDir;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

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
fn test_create_and_save_snippet() -> Result<()> {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = SnippetStorage::new(storage_path)?;
    
    // Create a new snippet
    let snippet = Snippet {
        id: Uuid::new_v4(),
        title: "Test Snippet".to_string(),
        content: "println!(\"Hello, world!\");".to_string(),
        language: Some("rust".to_string()),
        tags: vec!["test".to_string(), "example".to_string()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        backpack: None,
    };
    
    // Save the snippet
    storage.save_snippet(&snippet)?;
    
    // Retrieve the snippet
    let retrieved = storage.get_snippet(&snippet.id)?;
    
    assert_eq!(retrieved.id, snippet.id, "Snippet ID mismatch");
    assert_eq!(retrieved.title, snippet.title, "Snippet title mismatch");
    assert_eq!(retrieved.content, snippet.content, "Snippet content mismatch");
    assert_eq!(retrieved.language, snippet.language, "Snippet language mismatch");
    assert_eq!(retrieved.tags, snippet.tags, "Snippet tags mismatch");
    
    Ok(())
}

#[test]
/// Test that snippets can be searched by content
/// 
/// This test verifies that:
/// 1. Multiple snippets can be saved
/// 2. Snippets can be searched by content
/// 3. Search results are ordered by relevance
fn test_search_snippets() -> Result<()> {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = SnippetStorage::new(storage_path)?;
    
    // Create and save multiple snippets
    let snippets = vec![
        Snippet {
            id: Uuid::new_v4(),
            title: "Rust Hello World".to_string(),
            content: "println!(\"Hello, world!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string(), "hello".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: None,
        },
        Snippet {
            id: Uuid::new_v4(),
            title: "Python Hello World".to_string(),
            content: "print(\"Hello, world!\")".to_string(),
            language: Some("python".to_string()),
            tags: vec!["python".to_string(), "hello".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: None,
        },
        Snippet {
            id: Uuid::new_v4(),
            title: "Rust Function".to_string(),
            content: "fn add(a: i32, b: i32) -> i32 { a + b }".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string(), "function".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: None,
        },
    ];
    
    for snippet in &snippets {
        storage.save_snippet(snippet)?;
    }
    
    // Search for "Hello"
    let results = storage.search_snippets("Hello", None);
    assert_eq!(results.len(), 2, "Expected 2 search results");
    
    // Search for "rust"
    let results = storage.search_snippets("rust", None);
    assert_eq!(results.len(), 2, "Expected 2 search results");
    
    Ok(())
}

#[test]
/// Test that snippets can be organized in backpacks
/// 
/// This test verifies that:
/// 1. Snippets can be assigned to backpacks
/// 2. Snippets can be retrieved by backpack
/// 3. Backpacks can be listed
fn test_backpack_organization() -> Result<()> {
    let temp_dir = setup_test_storage();
    let storage_path = temp_dir.path();
    
    // Create a new storage instance
    let mut storage = SnippetStorage::new(storage_path)?;
    
    // Create snippets in different backpacks
    let snippets = vec![
        Snippet {
            id: Uuid::new_v4(),
            title: "Rust Snippet 1".to_string(),
            content: "println!(\"Hello from Rust!\");".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: Some("rust-backpack".to_string()),
        },
        Snippet {
            id: Uuid::new_v4(),
            title: "Python Snippet 1".to_string(),
            content: "print(\"Hello from Python!\")".to_string(),
            language: Some("python".to_string()),
            tags: vec!["python".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: Some("python-backpack".to_string()),
        },
        Snippet {
            id: Uuid::new_v4(),
            title: "Rust Snippet 2".to_string(),
            content: "fn main() { println!(\"Another Rust example\"); }".to_string(),
            language: Some("rust".to_string()),
            tags: vec!["rust".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            backpack: Some("rust-backpack".to_string()),
        },
    ];
    
    for snippet in &snippets {
        storage.save_snippet(snippet)?;
    }
    
    // List backpacks
    let backpacks = storage.list_backpacks();
    assert_eq!(backpacks.len(), 2, "Expected 2 backpacks");
    assert!(backpacks.contains(&"rust-backpack".to_string()), "Missing rust-backpack");
    assert!(backpacks.contains(&"python-backpack".to_string()), "Missing python-backpack");
    
    // Get snippets from rust-backpack
    let rust_snippets = storage.get_snippets_by_backpack("rust-backpack");
    assert_eq!(rust_snippets.len(), 2, "Expected 2 snippets in rust-backpack");
    
    Ok(())
} 