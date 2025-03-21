use anyhow::{Result, anyhow, Context};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;
use std::time::SystemTime;

use crate::models::ContentType;
use tempfile::NamedTempFile;

// Add clipboard module
pub mod clipboard;

// Add summarization module
pub mod summarization;

// Re-export clipboard functions for convenience
pub use clipboard::{read_clipboard, write_clipboard};

// Re-export summarization functions for convenience
pub use summarization::{summarize_text, SummaryMetadata};

/// Read content from a file
pub fn read_file_content(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|e| anyhow!("Failed to read file {}: {}", path.display(), e))
}

/// Read content from stdin
pub fn read_stdin_content() -> Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Open the system editor and return the content
pub fn open_editor(initial_content: Option<&str>) -> Result<String> {
    // Find the user's preferred editor
    let editor = get_editor()?;
    
    // Create a temporary file
    let mut temp_file = NamedTempFile::new()?;
    
    // Write initial content if provided
    if let Some(content) = initial_content {
        temp_file.write_all(content.as_bytes())?;
        temp_file.flush()?;
    }
    
    // Get the path to the temporary file
    let temp_path = temp_file.path().to_path_buf();
    
    // Open the editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;
    
    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status: {}", status));
    }
    
    // Read the content from the temporary file
    let content = fs::read_to_string(&temp_path)
        .with_context(|| format!("Failed to read from temporary file: {}", temp_path.display()))?;
    
    Ok(content)
}

/// Open the system editor with syntax highlighting hints based on content type
pub fn open_editor_with_type(content_type: ContentType, initial_content: Option<&str>) -> Result<String> {
    // Find the user's preferred editor
    let editor = get_editor()?;
    
    // Create a temporary file with appropriate extension
    let extension = match content_type {
        ContentType::Code => ".rs", // Default to Rust, but could be more specific
        ContentType::Text => ".txt",
        ContentType::Script => ".sh",
        ContentType::Other(ref lang) => {
            match lang.as_str() {
                "javascript" | "js" => ".js",
                "typescript" | "ts" => ".ts",
                "python" | "py" => ".py",
                "ruby" | "rb" => ".rb",
                "html" => ".html",
                "css" => ".css",
                "json" => ".json",
                "yaml" | "yml" => ".yml",
                "markdown" | "md" => ".md",
                "shell" | "sh" | "bash" => ".sh",
                "sql" => ".sql",
                _ => ".txt"
            }
        }
    };
    
    // Create a temporary file with appropriate extension
    let temp_dir = tempfile::tempdir()?;
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let file_name = format!("pocket_temp_{}{}", timestamp, extension);
    let temp_path = temp_dir.path().join(file_name);
    
    // Write initial content if provided
    if let Some(content) = initial_content {
        fs::write(&temp_path, content)?;
    } else {
        // Add template based on content type if no initial content
        let template = match content_type {
            ContentType::Code => match extension {
                ".rs" => "// Rust code snippet\n\nfn example() {\n    // Your code here\n}\n",
                ".js" => "// JavaScript code snippet\n\nfunction example() {\n    // Your code here\n}\n",
                ".ts" => "// TypeScript code snippet\n\nfunction example(): void {\n    // Your code here\n}\n",
                ".py" => "# Python code snippet\n\ndef example():\n    # Your code here\n    pass\n",
                ".rb" => "# Ruby code snippet\n\ndef example\n  # Your code here\nend\n",
                ".html" => "<!DOCTYPE html>\n<html>\n<head>\n    <title>Title</title>\n</head>\n<body>\n    <!-- Your content here -->\n</body>\n</html>\n",
                ".css" => "/* CSS snippet */\n\n.example {\n    /* Your styles here */\n}\n",
                ".json" => "{\n    \"key\": \"value\"\n}\n",
                ".yml" => "# YAML snippet\nkey: value\nnested:\n  subkey: subvalue\n",
                ".sh" => "#!/bin/bash\n\n# Your script here\necho \"Hello, world!\"\n",
                ".sql" => "-- SQL snippet\nSELECT * FROM table WHERE condition;\n",
                _ => "// Code snippet\n\n// Your code here\n"
            },
            ContentType::Text => "# Title\n\nYour text here...\n",
            ContentType::Script => "#!/bin/bash\n\n# Your script here\necho \"Hello, world!\"\n",
            ContentType::Other(_) => "# Content\n\nYour content here...\n"
        };
        fs::write(&temp_path, template)?;
    }
    
    // Open the editor
    let status = Command::new(&editor)
        .arg(&temp_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to open editor: {}", editor))?;
    
    if !status.success() {
        return Err(anyhow!("Editor exited with non-zero status: {}", status));
    }
    
    // Read the content from the temporary file
    let content = fs::read_to_string(&temp_path)
        .with_context(|| format!("Failed to read from temporary file: {}", temp_path.display()))?;
    
    Ok(content)
}

/// Edit an existing entry
pub fn edit_entry(id: &str, content: &str, content_type: ContentType) -> Result<String> {
    println!("Opening entry {} in editor. Make your changes and save to update.", id.cyan());
    open_editor_with_type(content_type, Some(content))
}

/// Get the user's preferred editor
fn get_editor() -> Result<String> {
    // Try to load from Pocket config first
    if let Ok(storage) = crate::storage::StorageManager::new() {
        if let Ok(config) = storage.load_config() {
            if !config.user.editor.is_empty() {
                return Ok(config.user.editor);
            }
        }
    }
    
    // Then try environment variables
    if let Ok(editor) = env::var("EDITOR") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }
    
    if let Ok(editor) = env::var("VISUAL") {
        if !editor.is_empty() {
            return Ok(editor);
        }
    }
    
    // Ask the user for their preferred editor
    println!("{}", "No preferred editor found in config or environment variables.".yellow());
    let editor = input::<String>("Please enter your preferred editor (e.g., vim, nano, code):", None)?;
    
    // Save the preference to config
    if let Ok(storage) = crate::storage::StorageManager::new() {
        if let Ok(mut config) = storage.load_config() {
            config.user.editor = editor.clone();
            let _ = storage.save_config(&config); // Ignore errors when saving config
        }
    }
    
    Ok(editor)
}

/// Detect content type from extension or content
pub fn detect_content_type(path: Option<&Path>, content: Option<&str>) -> ContentType {
    // Check file extension first if path is provided
    if let Some(path) = path {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" => return ContentType::Code,
                "go" => return ContentType::Code,
                "js" | "ts" => return ContentType::Code,
                "py" => return ContentType::Code,
                "java" => return ContentType::Code,
                "c" | "cpp" | "h" | "hpp" => return ContentType::Code,
                "cs" => return ContentType::Code,
                "rb" => return ContentType::Code,
                "php" => return ContentType::Code,
                "html" | "htm" => return ContentType::Other("html".to_string()),
                "css" => return ContentType::Other("css".to_string()),
                "json" => return ContentType::Other("json".to_string()),
                "yaml" | "yml" => return ContentType::Other("yaml".to_string()),
                "md" | "markdown" => return ContentType::Other("markdown".to_string()),
                "sql" => return ContentType::Other("sql".to_string()),
                "sh" | "bash" | "zsh" => return ContentType::Script,
                _ => {}
            }
        }
        
        // Check filename for specific patterns
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if filename.starts_with("Dockerfile") {
                return ContentType::Other("dockerfile".to_string());
            }
            
            if filename == "Makefile" || filename == "makefile" {
                return ContentType::Other("makefile".to_string());
            }
        }
    }
    
    // Check content if provided
    if let Some(content) = content {
        // Check for shebang line
        if content.starts_with("#!/bin/sh") || 
           content.starts_with("#!/bin/bash") || 
           content.starts_with("#!/usr/bin/env bash") ||
           content.starts_with("#!/bin/zsh") || 
           content.starts_with("#!/usr/bin/env zsh") {
            return ContentType::Script;
        }
        
        // Check for common code patterns
        let trimmed = content.trim();
        if trimmed.starts_with("#include") || trimmed.starts_with("#define") || 
           trimmed.starts_with("import ") || trimmed.starts_with("from ") || 
           trimmed.starts_with("package ") || trimmed.starts_with("using ") ||
           trimmed.starts_with("function ") || trimmed.starts_with("def ") ||
           trimmed.starts_with("class ") || trimmed.starts_with("struct ") ||
           trimmed.starts_with("enum ") || trimmed.starts_with("interface ") ||
           trimmed.contains("public class ") || trimmed.contains("private class ") ||
           trimmed.contains("fn ") || trimmed.contains("pub fn ") ||
           trimmed.contains("impl ") || trimmed.contains("trait ") {
            return ContentType::Code;
        }
        
        // Check for JSON
        if (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
           (trimmed.starts_with('[') && trimmed.ends_with(']')) {
            return ContentType::Other("json".to_string());
        }
        
        // Check for HTML
        if trimmed.starts_with("<!DOCTYPE html>") || 
           trimmed.starts_with("<html>") || 
           trimmed.contains("<body>") {
            return ContentType::Other("html".to_string());
        }
        
        // Check for Markdown
        if trimmed.starts_with("# ") || 
           trimmed.contains("\n## ") || 
           trimmed.contains("\n### ") {
            return ContentType::Other("markdown".to_string());
        }
    }
    
    // Default to text
    ContentType::Text
}

/// Prompt the user for confirmation
pub fn confirm(message: &str, default: bool) -> Result<bool> {
    Ok(Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()?)
}

/// Prompt the user for input
pub fn input<T>(message: &str, default: Option<T>) -> Result<T>
where
    T: std::str::FromStr + std::fmt::Display + Clone,
    T::Err: std::fmt::Display,
{
    let theme = ColorfulTheme::default();
    
    if let Some(default_value) = default {
        Ok(Input::<T>::with_theme(&theme)
            .with_prompt(message)
            .default(default_value)
            .interact()?)
    } else {
        Ok(Input::<T>::with_theme(&theme)
            .with_prompt(message)
            .interact()?)
    }
}

/// Prompt the user to select from a list of options
pub fn select<T>(message: &str, options: &[T]) -> Result<usize>
where
    T: std::fmt::Display,
{
    Ok(Select::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .items(options)
        .default(0)
        .interact()?)
}

/// Format content with tag
pub fn format_with_tag(tag: &str, content: &str) -> String {
    format!("--- {} ---\n{}\n--- end {} ---\n", tag, content, tag)
}

/// Truncate a string to a maximum length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut result = s.chars().take(max_len - 3).collect::<String>();
        result.push_str("...");
        result
    }
}

/// Extract the first line of a string
pub fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

/// Get a title from content (first line or truncated content)
pub fn get_title_from_content(content: &str) -> String {
    let first = first_line(content);
    if first.is_empty() {
        truncate_string(content, 50)
    } else {
        truncate_string(first, 50)
    }
}

/// Expand a path string with tilde and environment variables
pub fn expand_path(path: &str) -> Result<PathBuf> {
    let expanded = if path.starts_with("~") {
        if let Some(home) = dirs::home_dir() {
            let path_without_tilde = path.strip_prefix("~").unwrap_or("");
            home.join(path_without_tilde.strip_prefix("/").unwrap_or(path_without_tilde))
        } else {
            return Err(anyhow!("Could not determine home directory"));
        }
    } else {
        PathBuf::from(path)
    };

    // Expand environment variables
    let mut result = String::new();
    let mut in_var = false;
    let mut var_name = String::new();

    for c in expanded.to_str().unwrap_or(path).chars() {
        if in_var {
            if c.is_alphanumeric() || c == '_' {
                var_name.push(c);
            } else {
                if !var_name.is_empty() {
                    if let Ok(value) = std::env::var(&var_name) {
                        result.push_str(&value);
                    }
                    var_name.clear();
                } else {
                    result.push('$');
                }
                result.push(c);
                in_var = false;
            }
        } else if c == '$' {
            in_var = true;
        } else {
            result.push(c);
        }
    }

    if in_var && !var_name.is_empty() {
        if let Ok(value) = std::env::var(&var_name) {
            result.push_str(&value);
        }
    }

    Ok(PathBuf::from(result))
}

/// Find the cursor position in a file
/// This looks for a special marker like "// CURSOR" and returns its position
pub fn get_cursor_position(content: &str) -> Option<usize> {
    // First, look for a dedicated cursor marker
    for marker in ["// CURSOR", "# CURSOR", "<!-- CURSOR -->", "/* CURSOR */"] {
        if let Some(pos) = content.find(marker) {
            return Some(pos);
        }
    }
    
    // If no marker found, try to find a reasonable position
    // Look for two consecutive empty lines
    if let Some(pos) = content.find("\n\n\n") {
        return Some(pos + 2); // Position after the second newline
    }
    
    // Look for the end of the file
    Some(content.len())
} 