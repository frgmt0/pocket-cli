//! Integration tests for VCS and Snippet functionality
//! 
//! These tests verify that the VCS and Snippet components work correctly together.

use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    use pocket_cli::vcs::Repository;
    use pocket_cli::models::Snippet;
    use pocket_cli::storage::SnippetStorage;
    use crate::common::{create_temp_dir, create_test_file, create_test_snippet, assert_file_exists};
    use std::fs;
    use uuid::Uuid;
    
    #[test]
    /// Test that snippets can be versioned in a repository
    fn test_version_control_of_snippets() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        Repository::init(repo_path)?;
        
        // Create a snippets directory in the repository
        let snippets_dir = repo_path.join("snippets");
        fs::create_dir_all(&snippets_dir)?;
        
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
        create_test_file(&snippet_file, snippet_content)?;
        
        // Open the repository
        let mut repo = Repository::open(repo_path)?;
        
        // Add the snippet file to the repository
        repo.add_file(&snippet_file)?;
        
        // Create a commit (shove)
        let commit_id = repo.commit("Add hello world snippet", None)?;
        
        // Verify the commit exists
        let commit = repo.load_shove(&commit_id)?;
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
        
        create_test_file(&snippet_file, updated_snippet_content)?;
        
        // Add and commit the modified snippet
        repo.add_file(&snippet_file)?;
        let second_commit_id = repo.commit("Update hello world snippet", None)?;
        
        // Verify the second commit exists
        let second_commit = repo.load_shove(&second_commit_id)?;
        assert_eq!(second_commit.message, "Update hello world snippet", "Second commit message does not match");
        
        // Check out the first commit
        repo.checkout(&commit_id)?;
        
        // Verify the snippet file has the original content
        let file_content = fs::read_to_string(&snippet_file)?;
        assert!(file_content.contains("Hello World Snippet"), "Snippet file does not contain original title");
        assert!(file_content.contains("Hello, world!"), "Snippet file does not contain original content");
        
        // Check out the second commit
        repo.checkout(&second_commit_id)?;
        
        // Verify the snippet file has the updated content
        let file_content = fs::read_to_string(&snippet_file)?;
        assert!(file_content.contains("Updated Hello World Snippet"), "Snippet file does not contain updated title");
        assert!(file_content.contains("Hello, updated world!"), "Snippet file does not contain updated content");
        
        Ok(())
    }
    
    #[test]
    /// Test that workflows can be versioned in a repository
    fn test_version_control_of_workflows() -> Result<()> {
        let temp_dir = create_temp_dir();
        let repo_path = temp_dir.path();
        
        // Create a new repository
        Repository::init(repo_path)?;
        
        // Create a workflows directory in the repository
        let workflows_dir = repo_path.join("workflows");
        fs::create_dir_all(&workflows_dir)?;
        
        // Create a workflow file
        let workflow_content = r#"# Setup Python Project
# This workflow sets up a basic Python project structure

mkdir -p $1/src
mkdir -p $1/tests
touch $1/README.md
touch $1/requirements.txt
touch $1/setup.py
touch $1/src/__init__.py
touch $1/tests/__init__.py
"#;
        
        let workflow_file = workflows_dir.join("setup_python.pocket");
        create_test_file(&workflow_file, workflow_content)?;
        
        // Open the repository
        let mut repo = Repository::open(repo_path)?;
        
        // Add the workflow file to the repository
        repo.add_file(&workflow_file)?;
        
        // Create a commit (shove)
        let commit_id = repo.commit("Add Python setup workflow", None)?;
        
        // Verify the commit exists
        let commit = repo.load_shove(&commit_id)?;
        assert_eq!(commit.message, "Add Python setup workflow", "Commit message does not match");
        
        // Modify the workflow
        let updated_workflow_content = r#"# Setup Python Project
# This workflow sets up a basic Python project structure with virtual environment

mkdir -p $1/src
mkdir -p $1/tests
mkdir -p $1/docs
touch $1/README.md
touch $1/requirements.txt
touch $1/setup.py
touch $1/src/__init__.py
touch $1/tests/__init__.py
touch $1/docs/index.md

# Create virtual environment
python -m venv $1/venv
"#;
        
        create_test_file(&workflow_file, updated_workflow_content)?;
        
        // Add and commit the modified workflow
        repo.add_file(&workflow_file)?;
        let second_commit_id = repo.commit("Update Python setup workflow with venv", None)?;
        
        // Create a new timeline (branch)
        repo.create_timeline("feature-docs", Some(&second_commit_id))?;
        repo.switch_timeline("feature-docs")?;
        
        // Modify the workflow again on the new timeline
        let feature_workflow_content = r#"# Setup Python Project
# This workflow sets up a basic Python project structure with virtual environment and docs

mkdir -p $1/src
mkdir -p $1/tests
mkdir -p $1/docs/api
mkdir -p $1/docs/tutorials
touch $1/README.md
touch $1/requirements.txt
touch $1/setup.py
touch $1/src/__init__.py
touch $1/tests/__init__.py
touch $1/docs/index.md
touch $1/docs/api/index.md
touch $1/docs/tutorials/getting_started.md

# Create virtual environment
python -m venv $1/venv
"#;
        
        create_test_file(&workflow_file, feature_workflow_content)?;
        
        // Add and commit the modified workflow on the feature timeline
        repo.add_file(&workflow_file)?;
        let feature_commit_id = repo.commit("Enhance docs in Python setup workflow", None)?;
        
        // Switch back to the main timeline
        repo.switch_timeline("main")?;
        
        // Verify the workflow file has the content from the main timeline
        let file_content = fs::read_to_string(&workflow_file)?;
        assert!(file_content.contains("with virtual environment"), "Workflow file does not contain main timeline content");
        assert!(!file_content.contains("tutorials/getting_started.md"), "Workflow file should not contain feature timeline content");
        
        // Switch to the feature timeline
        repo.switch_timeline("feature-docs")?;
        
        // Verify the workflow file has the content from the feature timeline
        let file_content = fs::read_to_string(&workflow_file)?;
        assert!(file_content.contains("tutorials/getting_started.md"), "Workflow file does not contain feature timeline content");
        
        Ok(())
    }
} 