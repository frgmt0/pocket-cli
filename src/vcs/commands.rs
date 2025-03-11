//! Command handlers for Pocket VCS
//!
//! Implements the CLI commands for VCS operations.

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use colored::Colorize;
use glob;

use crate::vcs::{
    Repository, Timeline, Shove, ShoveId, Pile,
    ObjectStore, MergeResult, MergeStrategy
};
use crate::vcs::remote::RemoteManager;

/// Create a new repository
pub fn new_repo_command(path: &Path, template: Option<&str>, no_default: bool) -> Result<()> {
    println!("Creating new Pocket repository at {}", path.display());
    
    let repo = Repository::new(path)?;
    
    if !no_default {
        // Create default files like README.md and .pocketignore
        let readme_path = path.join("README.md");
        if !readme_path.exists() {
            std::fs::write(readme_path, "# New Pocket Repository\n\nCreated with Pocket VCS.\n")?;
        }
        
        let ignore_path = path.join(".pocketignore");
        if !ignore_path.exists() {
            std::fs::write(ignore_path, "# Pocket ignore file\n.DS_Store\n*.log\n")?;
        }
    }
    
    println!("Repository created successfully.");
    println!("Current timeline: {}", repo.current_timeline.name);
    
    Ok(())
}

/// Show repository status
pub fn status_command(path: &Path, verbose: bool) -> Result<()> {
    let repo = Repository::open(path)?;
    let status = repo.status()?;
    
    println!("On timeline: {}", status.current_timeline.bold());
    
    if let Some(head) = &status.head_shove {
        println!("Head shove: {}", head.as_str());
    } else {
        println!("No shoves yet");
    }
    
    if !status.piled_files.is_empty() {
        println!("\nChanges to be shoved:");
        for entry in &status.piled_files {
            let status_str = match &entry.status {
                crate::vcs::PileStatus::Added => "new file".green(),
                crate::vcs::PileStatus::Modified => "modified".yellow(),
                crate::vcs::PileStatus::Deleted => "deleted".red(),
                crate::vcs::PileStatus::Renamed(old_path) => format!("renamed from {}", old_path.display()).blue(),
            };
            println!("  {}: {}", status_str, entry.original_path.display());
        }
    }
    
    if !status.modified_files.is_empty() {
        println!("\nChanges not piled:");
        for path in &status.modified_files {
            println!("  {}: {}", "modified".red(), path.display());
        }
    }
    
    if !status.untracked_files.is_empty() {
        println!("\nUntracked files:");
        for path in &status.untracked_files {
            println!("  {}", path.display());
        }
    }
    
    if !status.conflicts.is_empty() {
        println!("\nConflicts:");
        for path in &status.conflicts {
            println!("  {}: {}", "conflict".bright_red().bold(), path.display());
        }
        println!("\nResolve conflicts and then run 'pocket shove' to complete the merge.");
    }
    
    Ok(())
}

/// Add files to the pile
pub fn pile_command(path: &Path, files: Vec<&Path>, all: bool, pattern: Option<&str>) -> Result<()> {
    let mut repo = Repository::open(path)?;
    
    if all {
        // Scan working directory for changes
        let status = repo.status()?;
        
        // Add modified files
        for file_path in &status.modified_files {
            println!("Adding modified file {} to the pile", file_path.display());
            repo.pile.add_path(file_path, &repo.object_store)?;
        }
        
        // Add untracked files
        for file_path in &status.untracked_files {
            println!("Adding untracked file {} to the pile", file_path.display());
            repo.pile.add_path(file_path, &repo.object_store)?;
        }
    } else if let Some(pat) = pattern {
        // Use glob pattern to find matching files
        let glob_pattern = if !pat.starts_with("./") {
            format!("./{}", pat)
        } else {
            pat.to_string()
        };
        
        for entry in glob::glob(&glob_pattern)? {
            match entry {
                Ok(path) => {
                    println!("Adding {} to the pile", path.display());
                    repo.pile.add_path(&path, &repo.object_store)?;
                }
                Err(e) => println!("Error matching pattern: {}", e),
            }
        }
    } else if !files.is_empty() {
        // Add specific files
        for file in files {
            println!("Adding {} to the pile", file.display());
            repo.pile.add_path(file, &repo.object_store)?;
        }
    } else {
        return Err(anyhow!("No files specified to add to the pile"));
    }
    
    // Save the pile
    let pile_path = repo.path.join(".pocket").join("piles").join("current.toml");
    repo.pile.save(&pile_path)?;
    
    Ok(())
}

/// Remove files from the pile
pub fn unpile_command(path: &Path, files: Vec<&Path>, all: bool) -> Result<()> {
    let mut repo = Repository::open(path)?;
    
    if all {
        println!("Removing all files from the pile");
        repo.pile.clear()?;
    } else if !files.is_empty() {
        for file in files {
            println!("Removing {} from the pile", file.display());
            repo.pile.remove_path(file)?;
        }
    } else {
        return Err(anyhow!("No files specified to remove from the pile"));
    }
    
    // Save the pile
    let pile_path = repo.path.join(".pocket").join("piles").join("current.toml");
    repo.pile.save(&pile_path)?;
    
    Ok(())
}

/// Create a shove (commit)
pub fn shove_command(path: &Path, message: Option<&str>, editor: bool) -> Result<()> {
    let mut repo = Repository::open(path)?;
    
    // Check if there are changes to commit
    if repo.pile.is_empty() {
        return Err(anyhow!("No changes to shove - pile is empty"));
    }
    
    // Get commit message
    let commit_msg = if editor {
        // Open editor for message
        let temp_file = std::env::temp_dir().join("pocket_shove_msg");
        if !temp_file.exists() {
            std::fs::write(&temp_file, "# Enter your shove message here\n")?;
        }
        
        let editor_cmd = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let status = std::process::Command::new(editor_cmd)
            .arg(&temp_file)
            .status()?;
            
        if !status.success() {
            return Err(anyhow!("Editor exited with non-zero status"));
        }
        
        let msg = std::fs::read_to_string(&temp_file)?;
        std::fs::remove_file(&temp_file)?;
        
        // Remove comments and trim
        msg.lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string()
    } else if let Some(msg) = message {
        msg.to_string()
    } else {
        return Err(anyhow!("No shove message provided. Use -m or -e to specify one"));
    };
    
    if commit_msg.is_empty() {
        return Err(anyhow!("Empty shove message"));
    }
    
    // Create the commit
    let shove_id = repo.create_shove(&commit_msg)?;
    println!("Created shove {} with message: {}", shove_id.as_str(), commit_msg);
    
    // Clear the pile after successful commit
    repo.pile.clear()?;
    let pile_path = repo.path.join(".pocket").join("piles").join("current.toml");
    repo.pile.save(&pile_path)?;
    
    Ok(())
}

/// Show shove history
pub fn log_command(path: &Path, graph: bool, limit: Option<usize>, timeline: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Get the timeline to show
    let timeline_name = timeline.unwrap_or(&repo.current_timeline.name);
    
    // Load the timeline
    let timeline_path = repo.path.join(".pocket").join("timelines").join(format!("{}.toml", timeline_name));
    let timeline = Timeline::load(&timeline_path)?;
    
    println!("Shove history for timeline '{}':", timeline_name);
    
    // Start from the head
    let mut current = timeline.head.clone();
    let mut count = 0;
    
    while let Some(shove_id) = current {
        // Check limit
        if let Some(lim) = limit {
            if count >= lim {
                break;
            }
        }
        
        // Load the shove
        let shove_path = repo.path.join(".pocket").join("shoves").join(format!("{}.toml", shove_id.as_str()));
        let shove = Shove::load(&shove_path)?;
        
        // Print shove info
        println!("\n{} {}", "Shove".yellow(), shove.id.as_str().bright_yellow());
        println!("Author: {} <{}>", shove.author.name, shove.author.email);
        println!("Date:   {}", shove.timestamp);
        println!("\n    {}", shove.message);
        
        // Move to parent
        if !shove.parent_ids.is_empty() {
            current = Some(shove.parent_ids[0].clone());
        } else {
            current = None;
        }
        
        count += 1;
    }
    
    Ok(())
}

/// Create a new timeline (branch)
pub fn timeline_new_command(path: &Path, name: &str, based_on: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Check if timeline already exists
    let timeline_path = repo.path.join(".pocket").join("timelines").join(format!("{}.toml", name));
    if timeline_path.exists() {
        return Err(anyhow!("Timeline '{}' already exists", name));
    }
    
    // Get the base shove
    let base_shove = if let Some(base) = based_on {
        // Use specified base
        ShoveId::from_str(base)?
    } else if let Some(head) = &repo.current_timeline.head {
        // Use current head
        head.clone()
    } else {
        // No base
        return Err(anyhow!("Cannot create timeline: no base shove specified and current timeline has no head"));
    };
    
    // Create the timeline
    let timeline = Timeline::new(name, Some(base_shove));
    
    // Save the timeline
    timeline.save(&timeline_path)?;
    
    println!("Created timeline '{}' based on shove {}", name, timeline.head.as_ref().unwrap().as_str());
    
    Ok(())
}

/// Switch to a timeline (branch)
pub fn timeline_switch_command(path: &Path, name: &str) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Check if timeline exists
    let timeline_path = repo.path.join(".pocket").join("timelines").join(format!("{}.toml", name));
    if !timeline_path.exists() {
        return Err(anyhow!("Timeline '{}' does not exist", name));
    }
    
    // Load the timeline
    let timeline = Timeline::load(&timeline_path)?;
    
    // Update HEAD
    let head_path = repo.path.join(".pocket").join("HEAD");
    std::fs::write(head_path, format!("timeline: {}\n", name))?;
    
    println!("Switched to timeline '{}'", name);
    
    Ok(())
}

/// List timelines (branches)
pub fn timeline_list_command(path: &Path) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Get all timeline files
    let timelines_dir = repo.path.join(".pocket").join("timelines");
    let entries = std::fs::read_dir(timelines_dir)?;
    
    println!("Timelines:");
    
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        
        if file_name_str.ends_with(".toml") {
            let timeline_name = file_name_str.trim_end_matches(".toml");
            
            // Mark current timeline
            if timeline_name == repo.current_timeline.name {
                println!("* {}", timeline_name.green());
            } else {
                println!("  {}", timeline_name);
            }
        }
    }
    
    Ok(())
}

/// Merge a timeline into the current timeline
pub fn merge_command(path: &Path, name: &str, strategy: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Check if timeline exists
    let timeline_path = repo.path.join(".pocket").join("timelines").join(format!("{}.toml", name));
    if !timeline_path.exists() {
        return Err(anyhow!("Timeline '{}' does not exist", name));
    }
    
    // Load the timeline
    let other_timeline = Timeline::load(&timeline_path)?;
    
    // Determine merge strategy
    let merge_strategy = match strategy {
        Some("fast-forward-only") => MergeStrategy::FastForwardOnly,
        Some("always-create-shove") => MergeStrategy::AlwaysCreateShove,
        Some("ours") => MergeStrategy::Ours,
        Some("theirs") => MergeStrategy::Theirs,
        _ => MergeStrategy::Auto,
    };
    
    // Create merger
    let merger = crate::vcs::merge::Merger::with_strategy(&repo, merge_strategy);
    
    // Perform merge
    let result = merger.merge_timeline(&other_timeline)?;
    
    if result.success {
        if result.fast_forward {
            println!("Fast-forward merge successful");
        } else {
            println!("Merge successful");
        }
        
        if let Some(shove_id) = result.shove_id {
            println!("Merge shove: {}", shove_id.as_str());
        }
    } else {
        println!("Merge failed");
        
        if !result.conflicts.is_empty() {
            println!("Conflicts:");
            for conflict in result.conflicts {
                println!("  {}", conflict.path.display());
            }
            println!("\nResolve conflicts and then run 'pocket shove' to complete the merge.");
        }
    }
    
    Ok(())
}

/// Add a remote repository
pub fn remote_add_command(path: &Path, name: &str, url: &str) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Create remote manager
    let mut remote_manager = RemoteManager::new(&repo)?;
    
    // Add remote
    remote_manager.add_remote(name, url)?;
    
    println!("Added remote '{}' with URL '{}'", name, url);
    
    Ok(())
}

/// Remove a remote repository
pub fn remote_remove_command(path: &Path, name: &str) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Create remote manager
    let mut remote_manager = RemoteManager::new(&repo)?;
    
    // Remove remote
    remote_manager.remove_remote(name)?;
    
    println!("Removed remote '{}'", name);
    
    Ok(())
}

/// List remote repositories
pub fn remote_list_command(path: &Path) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Create remote manager
    let remote_manager = RemoteManager::new(&repo)?;
    
    println!("Remotes:");
    
    for (name, remote) in &remote_manager.remotes {
        println!("  {}: {}", name, remote.url);
    }
    
    Ok(())
}

/// Fetch from a remote repository
pub fn fish_command(path: &Path, remote: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Create remote manager
    let remote_manager = RemoteManager::new(&repo)?;
    
    // Determine remote to fetch from
    let remote_name = if let Some(r) = remote {
        r
    } else if let Some(default) = &repo.config.remote.default_remote {
        default
    } else {
        return Err(anyhow!("No remote specified and no default remote configured"));
    };
    
    // Fetch from remote
    remote_manager.fetch(remote_name)?;
    
    println!("Fetched from remote '{}'", remote_name);
    
    Ok(())
}

/// Push to a remote repository
pub fn push_command(path: &Path, remote: Option<&str>, timeline: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    
    // Create remote manager
    let remote_manager = RemoteManager::new(&repo)?;
    
    // Determine remote to push to
    let remote_name = if let Some(r) = remote {
        r
    } else if let Some(default) = &repo.config.remote.default_remote {
        default
    } else {
        return Err(anyhow!("No remote specified and no default remote configured"));
    };
    
    // Determine timeline to push
    let timeline_name = timeline.unwrap_or(&repo.current_timeline.name);
    
    // Push to remote
    remote_manager.push(remote_name, timeline_name)?;
    
    println!("Pushed timeline '{}' to remote '{}'", timeline_name, remote_name);
    
    Ok(())
} 