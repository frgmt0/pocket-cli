use anyhow::{Result, anyhow};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

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

/// Open the default editor to edit content
pub fn open_editor(initial_content: Option<&str>) -> Result<String> {
    // For now, we'll just return a placeholder message
    // In a real implementation, this would open an editor
    if let Some(content) = initial_content {
        Ok(content.to_string())
    } else {
        Ok("This is a placeholder for editor content.".to_string())
    }
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

/// Format a string with color based on the tag
pub fn format_with_tag(tag: &str, content: &str) -> String {
    match tag {
        "+" => content.green().to_string(),
        "-" => content.red().to_string(),
        "!" => content.yellow().to_string(),
        _ => content.to_string(),
    }
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