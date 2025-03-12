//! Integration tests for VCS and Snippet functionality
//! 
//! These tests verify that the VCS and Snippet components work correctly together.

mod common;
use common::{create_temp_dir, setup_test_repository, create_test_file, assert_file_exists};

#[cfg(test)]
mod tests {
    use super::*;
    use pocket::vcs::Repository;
    use pocket::models::Snippet;
    use pocket::storage::SnippetStorage;
    use std::fs;
    use uuid::Uuid;
    
    #[test]
    /// Test that snippets can be versioned in a repository
    fn test_version_control_of_snippets() {
        let temp_dir = create_temp_dir();
        let repo_path = setup_test_repository(temp_dir.path());
        
        // Create a snippets directory in the repository
        let snippets_dir = repo_path.join("snippets");
        fs::create_dir_all(&snippets_dir).expect("Failed to create snippets directory");
        
        // Create a snippet file
        let snippet_content = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "title": "Hello World Snippet",
            "content": "println!(\"Hello, world!\");",
            "language": "rust",
            "tags": ["hello", "world"],
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-01T00:00:00Z"
        }"#;
        
        let snippet_file = snippets_dir.join("hello_world.json");
        create_test_file(&snippet_file, snippet_content);
        
        // Open the repository
        let mut repo = Repository::open(&repo_path).expect("Failed to open repository");
        
        // Add the snippet file to the repository
        repo.add_file(&snippet_file).expect("Failed to add snippet file");
        
        // Create a commit (shove)
        let commit_id = repo.commit("Add hello world snippet", None)
            .expect("Failed to commit changes");
        
        // Verify the commit exists
        let commit = repo.get_commit(&commit_id).expect("Failed to get commit");
        assert_eq!(commit.message, "Add hello world snippet", "Commit message does not match");
        
        // Modify the snippet
        let updated_snippet_content = r#"{
            "id": "12345678-1234-1234-1234-123456789012",
            "title": "Updated Hello World Snippet",
            "content": "println!(\"Hello, updated world!\");",
            "language": "rust",
            "tags": ["hello", "world", "updated"],
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-02T00:00:00Z"
        }"#;
        
        create_test_file(&snippet_file, updated_snippet_content);
        
        // Add and commit the modified snippet
        repo.add_file(&snippet_file).expect("Failed to add modified snippet file");
        let second_commit_id = repo.commit("Update hello world snippet", None)
            .expect("Failed to commit changes");
        
        // Verify the second commit exists
        let second_commit = repo.get_commit(&second_commit_id).expect("Failed to get second commit");
        assert_eq!(second_commit.message, "Update hello world snippet", "Second commit message does not match");
        
        // Check out the first commit
        repo.checkout(&commit_id).expect("Failed to checkout first commit");
        
        // Verify the snippet file has the original content
        let file_content = fs::read_to_string(&snippet_file).expect("Failed to read snippet file");
        assert!(file_content.contains("Hello World Snippet"), "Snippet file does not contain original title");
        assert!(file_content.contains("Hello, world!"), "Snippet file does not contain original content");
        
        // Check out the second commit
        repo.checkout(&second_commit_id).expect("Failed to checkout second commit");
        
        // Verify the snippet file has the updated content
        let file_content = fs::read_to_string(&snippet_file).expect("Failed to read snippet file");
        assert!(file_content.contains("Updated Hello World Snippet"), "Snippet file does not contain updated title");
        assert!(file_content.contains("Hello, updated world!"), "Snippet file does not contain updated content");
    }
} 