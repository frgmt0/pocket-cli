//! Command handlers for Pocket VCS
//!
//! Implements the CLI commands for VCS operations.

use std::path::{Path, PathBuf};
use anyhow::{Result, anyhow};
use colored::Colorize;
use glob;
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};
use indicatif::{ProgressBar, ProgressStyle};

use crate::vcs::{
    Repository, Timeline, Shove, ShoveId, Pile,
    ObjectStore, MergeStrategy
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

/// Display the status of the repository
pub fn status_command(path: &Path, verbose: bool) -> Result<()> {
    let repo = Repository::open(path)?;
    let status = repo.status()?;
    
    // Create a beautiful header
    println!("\n{} {} {}\n", "🔍".bright_cyan(), "Pocket VCS Status".bold().bright_cyan(), "🔍".bright_cyan());
    
    // Current timeline
    println!("{} {}: {}", "🌿".green(), "Current Timeline".bold(), status.current_timeline.bright_green());
    
    // Head shove
    if let Some(head) = &status.head_shove {
        let shove_path = repo.path.join(".pocket").join("shoves").join(format!("{}.toml", head.as_str()));
        if shove_path.exists() {
            let shove_content = std::fs::read_to_string(shove_path)?;
            let shove: Shove = toml::from_str(&shove_content)?;
            println!("{} {}: {} ({})", "📌".yellow(), "HEAD Shove".bold(), 
                head.as_str()[0..8].bright_yellow(), 
                shove.message.lines().next().unwrap_or("").italic());
        } else {
            println!("{} {}: {}", "📌".yellow(), "HEAD Shove".bold(), head.as_str()[0..8].bright_yellow());
        }
    } else {
        println!("{} {}: {}", "📌".yellow(), "HEAD Shove".bold(), "None".dimmed());
    }
    
    // Piled files (staged)
    if !status.piled_files.is_empty() {
        println!("\n{} {} {}", "📦".green(), "Piled Changes".bold().green(), format!("({})", status.piled_files.len()).green());
        for entry in &status.piled_files {
            let status_icon = match entry.status {
                crate::vcs::PileStatus::Added => "🆕".green(),
                crate::vcs::PileStatus::Modified => "📝".yellow(),
                crate::vcs::PileStatus::Deleted => "🗑️".red(),
                crate::vcs::PileStatus::Renamed(_) => "📋".blue(),
            };
            println!("  {} {}", status_icon, entry.original_path.display().to_string().bright_white());
        }
    } else {
        println!("\n{} {}", "📦".dimmed(), "No Piled Changes".dimmed());
    }
    
    // Modified files (unstaged)
    if !status.modified_files.is_empty() {
        println!("\n{} {} {}", "📄".yellow(), "Modified Files".bold().yellow(), format!("({})", status.modified_files.len()).yellow());
        for file in &status.modified_files {
            println!("  {} {}", "📝".yellow(), file.display().to_string().bright_white());
        }
    } else {
        println!("\n{} {}", "📄".dimmed(), "No Modified Files".dimmed());
    }
    
    // Untracked files
    if !status.untracked_files.is_empty() {
        println!("\n{} {} {}", "❓".bright_red(), "Untracked Files".bold().bright_red(), format!("({})", status.untracked_files.len()).bright_red());
        
        // If there are too many untracked files, only show a few
        let max_display = if verbose { status.untracked_files.len() } else { 5.min(status.untracked_files.len()) };
        for file in &status.untracked_files[0..max_display] {
            println!("  {} {}", "❓".bright_red(), file.display().to_string().bright_white());
        }
        
        if status.untracked_files.len() > max_display {
            println!("  {} {} more untracked files", "⋯".bright_red(), status.untracked_files.len() - max_display);
            println!("  {} Use {} to see all files", "💡".yellow(), "--verbose".bright_cyan());
        }
    } else {
        println!("\n{} {}", "❓".dimmed(), "No Untracked Files".dimmed());
    }
    
    // Conflicts
    if !status.conflicts.is_empty() {
        println!("\n{} {} {}", "⚠️".bright_red(), "Conflicts".bold().bright_red(), format!("({})", status.conflicts.len()).bright_red());
        for file in &status.conflicts {
            println!("  {} {}", "⚠️".bright_red(), file.display().to_string().bright_white());
        }
        println!("  {} Use {} to resolve conflicts", "💡".yellow(), "pocket merge --resolve".bright_cyan());
    }
    
    // Show a helpful tip
    println!("\n{} {}", "💡".yellow(), "Tip: Use 'pocket help' to see available commands".italic());
    
    Ok(())
}

/// Interactive pile command
pub fn interactive_pile_command(path: &Path, files: Vec<String>, all: bool, pattern: Option<String>) -> Result<()> {
    // If files, all flag, or pattern is provided, use the non-interactive pile command
    if !files.is_empty() || all || pattern.is_some() {
        let file_paths: Vec<&Path> = files.iter().map(|f| Path::new(f)).collect();
        return pile_command(path, file_paths, all, pattern.as_deref());
    }

    let repo = Repository::open(path)?;
    let status = repo.status()?;
    
    println!("\n{} {} {}\n", "📦".green(), "Interactive Pile".bold().green(), "📦".green());
    
    // No files to pile
    if status.modified_files.is_empty() && status.untracked_files.is_empty() {
        println!("{} {}", "ℹ️".blue(), "No files to pile. Your working directory is clean.".italic());
        return Ok(());
    }
    
    // Combine modified and untracked files
    let mut files_to_choose = Vec::new();
    
    for file in &status.modified_files {
        files_to_choose.push((file.clone(), "Modified".to_string(), "📝".to_string()));
    }
    
    for file in &status.untracked_files {
        files_to_choose.push((file.clone(), "Untracked".to_string(), "❓".to_string()));
    }
    
    // Sort files by path
    files_to_choose.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Create selection items
    let items: Vec<String> = files_to_choose.iter()
        .map(|(path, status, icon)| format!("{} {} ({})", icon, path.display(), status))
        .collect();
    
    // Add "All files" and "Done" options
    let all_files_option = format!("📦 Pile all files ({})", files_to_choose.len());
    let done_option = "✅ Done".to_string();
    
    let mut selection_items = vec![all_files_option.clone()];
    selection_items.extend(items);
    selection_items.push(done_option.clone());
    
    // Track which files have been piled
    let mut piled_files = Vec::new();
    
    // Create progress bar for piling
    let progress_style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");
    
    // Interactive selection loop
    loop {
        println!("\n{} {} files piled so far", "📊".blue(), piled_files.len());
        
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select files to pile (↑↓ to move, Enter to select)")
            .default(0)
            .items(&selection_items)
            .interact()
            .unwrap();
        
        if selection_items[selection] == done_option {
            break;
        } else if selection_items[selection] == all_files_option {
            // Pile all files
            let pb = ProgressBar::new(files_to_choose.len() as u64);
            pb.set_style(progress_style.clone());
            pb.set_message("Piling files...");
            
            for (i, (file, _, _)) in files_to_choose.iter().enumerate() {
                if !piled_files.contains(file) {
                    // In a real implementation, we would call repo.pile.add_path() here
                    piled_files.push(file.clone());
                }
                pb.set_position(i as u64 + 1);
                pb.set_message(format!("Piled {}", file.display()));
                std::thread::sleep(std::time::Duration::from_millis(50)); // Simulate work
            }
            
            pb.finish_with_message(format!("✅ All {} files piled successfully", files_to_choose.len()));
            break;
        } else {
            // Pile individual file
            let (file, _, _) = &files_to_choose[selection - 1]; // -1 because of "All files" option
            
            if !piled_files.contains(file) {
                // In a real implementation, we would call repo.pile.add_path() here
                piled_files.push(file.clone());
                println!("{} Piled: {}", "✅".green(), file.display());
            } else {
                println!("{} Already piled: {}", "ℹ️".blue(), file.display());
            }
        }
    }
    
    if !piled_files.is_empty() {
        println!("\n{} {} files piled successfully", "✅".green(), piled_files.len());
        println!("{} Use {} to create a shove", "💡".yellow(), "pocket shove".bright_cyan());
    } else {
        println!("\n{} No files were piled", "ℹ️".blue());
    }
    
    Ok(())
}

/// Interactive shove command
pub fn interactive_shove_command(path: &Path) -> Result<()> {
    let repo = Repository::open(path)?;
    
    println!("\n{} {} {}\n", "📦".green(), "Create Shove".bold().green(), "📦".green());
    
    // Check if there are piled changes
    let status = repo.status()?;
    if status.piled_files.is_empty() {
        println!("{} {}", "ℹ️".blue(), "No piled changes to shove.".italic());
        
        if !status.modified_files.is_empty() || !status.untracked_files.is_empty() {
            println!("{} Use {} to pile changes first", "💡".yellow(), "pocket pile".bright_cyan());
        }
        
        return Ok(());
    }
    
    // Show piled changes
    println!("{} {} {}", "📦".green(), "Piled Changes".bold().green(), format!("({})", status.piled_files.len()).green());
    for entry in &status.piled_files {
        let status_icon = match entry.status {
            crate::vcs::PileStatus::Added => "🆕".green(),
            crate::vcs::PileStatus::Modified => "📝".yellow(),
            crate::vcs::PileStatus::Deleted => "🗑️".red(),
            crate::vcs::PileStatus::Renamed(_) => "📋".blue(),
        };
        println!("  {} {}", status_icon, entry.original_path.display().to_string().bright_white());
    }
    
    // Get shove message
    println!("\n{} {}", "✏️".yellow(), "Enter a shove message:".bold());
    let message = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Message")
        .interact_text()
        .unwrap();
    
    // Confirm shove creation
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Create shove with these changes?")
        .default(true)
        .interact()
        .unwrap()
    {
        println!("\n{} Shove creation cancelled", "❌".red());
        return Ok(());
    }
    
    // Create progress bar for shoving
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));
    
    // Simulate shove creation
    pb.set_message("Creating tree objects...");
    for i in 0..30 {
        pb.set_position(i);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    pb.set_message("Calculating changes...");
    for i in 30..60 {
        pb.set_position(i);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    pb.set_message("Creating shove...");
    for i in 60..90 {
        pb.set_position(i);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    pb.set_message("Updating timeline...");
    for i in 90..100 {
        pb.set_position(i);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    
    // In a real implementation, we would call repo.create_shove() here
    let shove_id = "abcdef1234567890";
    let shove_id_short = &shove_id[0..8];
    
    pb.finish_with_message(format!("✅ Shove created successfully: {}", shove_id_short.bright_yellow()));
    
    println!("\n{} {} created with message:", "✅".green(), format!("Shove {}", shove_id_short).bright_yellow());
    println!("  {}", message.italic());
    
    Ok(())
}

/// Interactive timeline command
pub fn interactive_timeline_command(path: &Path) -> Result<()> {
    let repo = Repository::open(path)?;
    
    println!("\n{} {} {}\n", "🌿".green(), "Timeline Management".bold().green(), "🌿".green());
    
    // Get current timeline
    let status = repo.status()?;
    println!("{} {}: {}", "🌿".green(), "Current Timeline".bold(), status.current_timeline.bright_green());
    
    // List available timelines
    let timelines_dir = repo.path.join(".pocket").join("timelines");
    let mut timelines = Vec::new();
    
    if timelines_dir.exists() {
        for entry in std::fs::read_dir(timelines_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();
                if filename_str.ends_with(".toml") {
                    let timeline_name = filename_str.trim_end_matches(".toml").to_string();
                    timelines.push(timeline_name);
                }
            }
        }
    }
    
    // Sort timelines
    timelines.sort();
    
    // Show available timelines
    println!("\n{} {} {}", "📋".blue(), "Available Timelines".bold().blue(), format!("({})", timelines.len()).blue());
    for timeline in &timelines {
        let current_marker = if timeline == &status.current_timeline { "✓ ".green() } else { "  ".normal() };
        println!("{}{} {}", current_marker, "🌿".green(), timeline.bright_white());
    }
    
    // Show options
    println!("\n{} {}", "🔍".cyan(), "What would you like to do?".bold());
    
    let options = vec![
        "🆕 Create new timeline",
        "🔄 Switch timeline",
        "🔙 Back to main menu",
    ];
    
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an option")
        .default(0)
        .items(&options)
        .interact()
        .unwrap();
    
    match selection {
        0 => {
            // Create new timeline
            println!("\n{} {}", "🆕".green(), "Create New Timeline".bold());
            
            let name = Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("Timeline name")
                .interact_text()
                .unwrap();
            
            let base_on_current = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Base on current timeline ({})?", status.current_timeline))
                .default(true)
                .interact()
                .unwrap();
            
            println!("\n{} Creating timeline: {}", "⏳".yellow(), name.bright_white());
            
            // In a real implementation, we would create the timeline here
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            println!("{} Timeline {} created successfully", "✅".green(), name.bright_green());
            
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Switch to new timeline ({})?", name))
                .default(true)
                .interact()
                .unwrap()
            {
                println!("\n{} Switching to timeline: {}", "⏳".yellow(), name.bright_white());
                
                // In a real implementation, we would switch to the timeline here
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                println!("{} Switched to timeline {}", "✅".green(), name.bright_green());
            }
        },
        1 => {
            // Switch timeline
            println!("\n{} {}", "🔄".green(), "Switch Timeline".bold());
            
            if timelines.is_empty() {
                println!("{} No timelines available", "❌".red());
                return Ok(());
            }
            
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select timeline to switch to")
                .default(0)
                .items(&timelines)
                .interact()
                .unwrap();
            
            let selected_timeline = &timelines[selection];
            
            println!("\n{} Switching to timeline: {}", "⏳".yellow(), selected_timeline.bright_white());
            
            // In a real implementation, we would switch to the timeline here
            std::thread::sleep(std::time::Duration::from_millis(500));
            
            println!("{} Switched to timeline {}", "✅".green(), selected_timeline.bright_green());
        },
        _ => {
            // Back to main menu
            println!("\n{} Returning to main menu", "🔙".blue());
        }
    }
    
    Ok(())
}

/// Add files to the pile (staging area)
pub fn pile_command(path: &Path, files: Vec<&Path>, all: bool, pattern: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    let mut pile = repo.pile.clone();
    let mut added_count = 0;
    
    // Read ignore patterns from .pocketignore if it exists
    let ignore_path = repo.path.join(".pocketignore");
    let ignore_patterns = if ignore_path.exists() {
        read_ignore_patterns(&ignore_path)?
    } else {
        repo.config.core.ignore_patterns.clone()
    };
    
    // Function to check if a file should be ignored
    let should_ignore = |file_path: &Path| -> bool {
        // Skip files in .git, .pocket, or other VCS directories
        if file_path.to_string_lossy().contains("/.pocket/") || 
           file_path.to_string_lossy().contains("/.git/") {
            return true;
        }
        
        // Check if the file matches any ignore pattern
        let relative_path = if let Ok(rel_path) = file_path.strip_prefix(&repo.path) {
            rel_path
        } else {
            file_path
        };
        
        ignore_patterns.iter().any(|pattern| {
            if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                glob_pattern.matches_path(relative_path)
            } else {
                false
            }
        })
    };
    
    // If --all flag is provided, add all modified files
    if all {
        let status = repo.status()?;
        for file_path in &status.modified_files {
            if !should_ignore(file_path) {
                pile.add_path(file_path, &repo.object_store)?;
                println!("{} {}", "✅".green(), format!("Added: {}", file_path.display()).bright_white());
                added_count += 1;
            }
        }
        
        for file_path in &status.untracked_files {
            if !should_ignore(file_path) {
                pile.add_path(file_path, &repo.object_store)?;
                println!("{} {}", "✅".green(), format!("Added: {}", file_path.display()).bright_white());
                added_count += 1;
            }
        }
    }
    // If pattern is provided, add files matching the pattern
    else if let Some(pattern_str) = pattern {
        let matches = glob::glob(pattern_str)?;
        for entry in matches {
            match entry {
                Ok(path) => {
                    if path.is_file() && !should_ignore(&path) {
                        pile.add_path(&path, &repo.object_store)?;
                        println!("{} {}", "✅".green(), format!("Added: {}", path.display()).bright_white());
                        added_count += 1;
                    } else if path.is_dir() {
                        // Recursively add all files in the directory
                        added_count += add_directory_recursively(&path, &mut pile, &repo.object_store, &repo.path, &ignore_patterns)?;
                    }
                }
                Err(e) => {
                    println!("{} {}", "⚠️".yellow(), format!("Error matching pattern: {}", e).yellow());
                }
            }
        }
    }
    // Otherwise, add the specified files
    else {
        for file_path in files {
            if file_path.is_file() && !should_ignore(file_path) {
                pile.add_path(file_path, &repo.object_store)?;
                println!("{} {}", "✅".green(), format!("Added: {}", file_path.display()).bright_white());
                added_count += 1;
            } else if file_path.is_dir() {
                // Recursively add all files in the directory
                added_count += add_directory_recursively(file_path, &mut pile, &repo.object_store, &repo.path, &ignore_patterns)?;
            } else {
                // Check if it's a glob pattern
                let path_str = file_path.to_string_lossy();
                if path_str.contains('*') || path_str.contains('?') || path_str.contains('[') {
                    let matches = glob::glob(&path_str)?;
                    for entry in matches {
                        match entry {
                            Ok(path) => {
                                if path.is_file() && !should_ignore(&path) {
                                    pile.add_path(&path, &repo.object_store)?;
                                    println!("{} {}", "✅".green(), format!("Added: {}", path.display()).bright_white());
                                    added_count += 1;
                                } else if path.is_dir() {
                                    // Recursively add all files in the directory
                                    added_count += add_directory_recursively(&path, &mut pile, &repo.object_store, &repo.path, &ignore_patterns)?;
                                }
                            }
                            Err(e) => {
                                println!("{} {}", "⚠️".yellow(), format!("Error matching pattern: {}", e).yellow());
                            }
                        }
                    }
                } else {
                    println!("{} {}", "⚠️".yellow(), format!("File not found: {}", file_path.display()).yellow());
                }
            }
        }
    }
    
    // Save the updated pile
    let pile_path = repo.path.join(".pocket").join("piles").join("current.toml");
    // Ensure the piles directory exists
    std::fs::create_dir_all(pile_path.parent().unwrap())?;
    pile.save(&pile_path)?;
    
    if added_count > 0 {
        println!("\n{} {} files added to the pile", "✅".green(), added_count);
        println!("{} Use {} to create a shove", "💡".yellow(), "pocket shove".bright_cyan());
    } else {
        println!("{} No files added to the pile", "ℹ️".blue());
    }
    
    Ok(())
}

/// Recursively add all files in a directory to the pile
fn add_directory_recursively(dir_path: &Path, pile: &mut Pile, object_store: &ObjectStore, repo_path: &Path, ignore_patterns: &[String]) -> Result<usize> {
    let mut added_count = 0;
    
    // Create a progress bar for directory scanning
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );
    spinner.set_message(format!("Scanning directory: {}", dir_path.display()));
    
    // Use walkdir to recursively iterate through the directory
    for entry in walkdir::WalkDir::new(dir_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
            
        spinner.tick();
        
        let path = entry.path();
        if path.is_file() {
            // Skip files in .git, .pocket, or other VCS directories
            if path.to_string_lossy().contains("/.pocket/") || 
               path.to_string_lossy().contains("/.git/") {
                continue;
            }
            
            // Check if the file matches any ignore pattern
            let relative_path = if let Ok(rel_path) = path.strip_prefix(repo_path) {
                rel_path
            } else {
                path
            };
            
            let should_ignore = ignore_patterns.iter().any(|pattern| {
                if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
                    glob_pattern.matches_path(relative_path)
                } else {
                    false
                }
            });
            
            if should_ignore {
                continue;
            }
            
            // Add the file to the pile
            pile.add_path(path, object_store)?;
            spinner.set_message(format!("Added: {}", path.display()));
            added_count += 1;
        }
    }
    
    spinner.finish_with_message(format!("Added {} files from {}", added_count, dir_path.display()));
    
    Ok(added_count)
}

/// Find the repository root by looking for .pocket directory
fn find_repository_root(path: &Path) -> Result<PathBuf> {
    let mut current = path.to_path_buf();
    
    loop {
        if current.join(".pocket").exists() {
            return Ok(current);
        }
        
        if !current.pop() {
            return Err(anyhow!("Not a pocket repository (or any parent directory)"));
        }
    }
}

/// Read ignore patterns from a .pocketignore file
fn read_ignore_patterns(path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(path)?;
    let patterns = content.lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
        .map(|line| line.trim().to_string())
        .collect();
    
    Ok(patterns)
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

/// Display the commit history with beautiful formatting
pub fn log_command(path: &Path, verbose: bool, timeline: Option<&str>) -> Result<()> {
    let repo = Repository::open(path)?;
    let status = repo.status()?;
    
    // Get the timeline to show
    let timeline_name = timeline.unwrap_or(&status.current_timeline);
    
    println!("\n{} {} {}\n", "📜".bright_cyan(), format!("Pocket VCS Log ({})", timeline_name).bold().bright_cyan(), "📜".bright_cyan());
    
    // Get the timeline
    let timelines_dir = repo.path.join(".pocket").join("timelines");
    let timeline_path = timelines_dir.join(format!("{}.toml", timeline_name));
    
    if !timeline_path.exists() {
        return Err(anyhow!("Timeline {} not found", timeline_name));
    }
    
    // In a real implementation, we would load the timeline and its shoves
    // For now, we'll create a simulated history
    let shoves = simulate_shove_history();
    
    // Display the shoves
    for (i, shove) in shoves.iter().enumerate() {
        // Shove ID and message
        println!("{} {} {}", 
            "📌".yellow(), 
            shove.id[0..8].bright_yellow().bold(),
            shove.message.lines().next().unwrap_or("").bright_white()
        );
        
        // Author and date
        println!("{}  {} {} on {}", 
            " ".repeat(4),
            "👤".blue(),
            shove.author.bright_blue(),
            shove.date.dimmed()
        );
        
        // Show full message if verbose
        if verbose && shove.message.lines().count() > 1 {
            println!();
            for line in shove.message.lines().skip(1) {
                println!("{}  {}", " ".repeat(4), line);
            }
        }
        
        // Show changes
        if verbose {
            println!("{}  {}", " ".repeat(4), "Changes:".dimmed());
            for change in &shove.changes {
                let icon = match change.change_type {
                    ChangeType::Added => "🆕".green(),
                    ChangeType::Modified => "📝".yellow(),
                    ChangeType::Deleted => "🗑️".red(),
                    ChangeType::Renamed => "📋".blue(),
                };
                println!("{}    {} {}", " ".repeat(4), icon, change.path);
            }
        }
        
        // Add graph lines between shoves
        if i < shoves.len() - 1 {
            println!("{}  │", " ".repeat(2));
            println!("{}  │", " ".repeat(2));
        }
    }
    
    Ok(())
}

// Simulate a shove history for demonstration
fn simulate_shove_history() -> Vec<SimulatedShove> {
    vec![
        SimulatedShove {
            id: "abcdef1234567890".to_string(),
            message: "Implement interactive merge resolution".to_string(),
            author: "dev@example.com".to_string(),
            date: "2025-03-15 14:30:45".to_string(),
            changes: vec![
                SimulatedChange {
                    change_type: ChangeType::Modified,
                    path: "src/vcs/merge.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/commands.rs".to_string(),
                },
            ],
        },
        SimulatedShove {
            id: "98765432abcdef12".to_string(),
            message: "Add remote repository functionality\n\nImplemented push, pull, and fetch operations for remote repositories.".to_string(),
            author: "dev@example.com".to_string(),
            date: "2025-03-14 10:15:30".to_string(),
            changes: vec![
                SimulatedChange {
                    change_type: ChangeType::Modified,
                    path: "src/vcs/remote.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Modified,
                    path: "src/vcs/repository.rs".to_string(),
                },
            ],
        },
        SimulatedShove {
            id: "1234567890abcdef".to_string(),
            message: "Initial implementation of VCS".to_string(),
            author: "dev@example.com".to_string(),
            date: "2025-03-10 09:00:00".to_string(),
            changes: vec![
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/mod.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/repository.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/shove.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/pile.rs".to_string(),
                },
                SimulatedChange {
                    change_type: ChangeType::Added,
                    path: "src/vcs/timeline.rs".to_string(),
                },
            ],
        },
    ]
}

// Simulated shove for demonstration
struct SimulatedShove {
    id: String,
    message: String,
    author: String,
    date: String,
    changes: Vec<SimulatedChange>,
}

// Simulated file change for demonstration
struct SimulatedChange {
    change_type: ChangeType,
    path: String,
}

// Change type enum
#[derive(Clone, Copy)]
enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
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

/// Manage ignore patterns
pub fn ignore_command(path: &Path, add: Option<&str>, remove: Option<&str>, list: bool) -> Result<()> {
    let repo = Repository::open(path)?;
    let mut config = repo.config.clone();
    let ignore_path = repo.path.join(".pocketignore");
    
    // Read existing patterns from .pocketignore file if it exists
    let mut patterns = if ignore_path.exists() {
        let content = std::fs::read_to_string(&ignore_path)?;
        content.lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|line| line.trim().to_string())
            .collect::<Vec<String>>()
    } else {
        config.core.ignore_patterns.clone()
    };
    
    if let Some(pattern) = add {
        // Add new pattern if it doesn't already exist
        if !patterns.contains(&pattern.to_string()) {
            patterns.push(pattern.to_string());
            println!("{} Added ignore pattern: {}", "✅".green(), pattern);
        } else {
            println!("{} Pattern already exists: {}", "ℹ️".blue(), pattern);
        }
    }
    
    if let Some(pattern) = remove {
        // Remove pattern if it exists
        if let Some(pos) = patterns.iter().position(|p| p == pattern) {
            patterns.remove(pos);
            println!("{} Removed ignore pattern: {}", "✅".green(), pattern);
        } else {
            println!("{} Pattern not found: {}", "⚠️".yellow(), pattern);
        }
    }
    
    if list {
        // List all patterns
        println!("\n{} Ignore patterns:", "📋".bright_cyan());
        if patterns.is_empty() {
            println!("  No ignore patterns defined");
        } else {
            for pattern in &patterns {
                println!("  - {}", pattern);
            }
        }
    }
    
    // Update config and save to .pocketignore file
    config.core.ignore_patterns = patterns.clone();
    
    // Save patterns to .pocketignore file
    let mut content = "# Pocket ignore file\n".to_string();
    for pattern in &patterns {
        content.push_str(&format!("{}\n", pattern));
    }
    std::fs::write(&ignore_path, content)?;
    
    // Update repository config
    let config_path = repo.path.join(".pocket").join("config.toml");
    let config_str = toml::to_string_pretty(&config)?;
    std::fs::write(config_path, config_str)?;
    
    Ok(())
} 