use crate::models::{Entry, Backpack, ContentType, Workflow, WorkflowCommand};
use crate::storage::StorageManager;
use crate::search::SearchEngine;
use crate::utils;
use anyhow::{Result, anyhow, Context};
use std::path::Path;
use owo_colors::OwoColorize;
use std::process::Command;
use dialoguer::Confirm;
use std::fs;

/// Add content to pocket storage
pub fn add_command(
    file: Option<String>,
    message: Option<String>,
    editor: bool,
    backpack: Option<String>,
) -> Result<String> {
    let storage = StorageManager::new()?;
    
    // Get content from file, message, or editor
    let content = if let Some(file_path) = file {
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(anyhow!("File not found: {}", file_path));
        }
        
        // Detect content type from file
        let content_type = utils::detect_content_type(Some(path), None);
        let content = utils::read_file_content(path)?;
        
        // Create and save entry
        let title = content.lines().next().unwrap_or("").to_string();
        let entry = Entry::new(title, content_type, Some(file_path), vec![]);
        
        storage.save_entry(&entry, &content, backpack.as_deref())?;
        
        entry.id
    } else if let Some(message) = message {
        // Detect content type from message
        let content_type = utils::detect_content_type(None, Some(&message));
        
        // Create and save entry
        let title = message.lines().next().unwrap_or("").to_string();
        let entry = Entry::new(title, content_type, None, vec![]);
        
        storage.save_entry(&entry, &message, backpack.as_deref())?;
        
        entry.id
    } else if editor {
        // Open editor for user to enter content
        println!("Opening editor. Write your content and save it to add it to Pocket.");
        
        // First detect a reasonable default content type based on backpack name
        let default_content_type = if let Some(backpack_name) = &backpack {
            match backpack_name.to_lowercase().as_str() {
                "rust" | "go" | "js" | "javascript" | "ts" | "typescript" | "py" | "python" | 
                "java" | "c" | "cpp" | "cs" | "csharp" => ContentType::Code,
                "html" | "css" | "web" => ContentType::Other(backpack_name.to_lowercase()),
                "markdown" | "md" | "docs" => ContentType::Other("markdown".to_string()),
                "sql" | "database" => ContentType::Other("sql".to_string()),
                "json" | "yaml" | "config" => ContentType::Other(backpack_name.to_lowercase()),
                _ => ContentType::Text,
            }
        } else {
            ContentType::Text
        };
        
        // Open editor with appropriate syntax highlighting
        let content = utils::open_editor_with_type(default_content_type, None)?;
        
        if content.trim().is_empty() {
            return Err(anyhow!("Empty content, nothing to save"));
        }
        
        // Detect content type from what was entered
        let content_type = utils::detect_content_type(None, Some(&content));
        
        // Create and save entry
        let title = content.lines().next().unwrap_or("").to_string();
        let entry = Entry::new(title, content_type, None, vec![]);
        
        storage.save_entry(&entry, &content, backpack.as_deref())?;
        
        entry.id
    } else {
        return Err(anyhow!("No content provided. Use --file, --message, or --editor"));
    };
    
    Ok(content)
}

/// List entries in pocket storage
pub fn list_command(
    include_backpacks: bool,
    backpack: Option<String>,
    json: bool,
) -> Result<()> {
    let storage = StorageManager::new()?;
    
    if let Some(backpack_name) = backpack {
        // List entries in a specific backpack
        let entries = storage.list_entries(Some(&backpack_name))?;
        if entries.is_empty() {
            println!("No entries found in backpack '{}'", backpack_name);
            return Ok(());
        }
        
        if json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        } else {
            println!("Entries in backpack '{}':", backpack_name);
            for entry in entries {
                println!("  {} - {}", entry.id, entry.title);
            }
        }
    } else if include_backpacks {
        // List entries in all backpacks
        let backpacks = storage.list_backpacks()?;
        let general_entries = storage.list_entries(None)?;
        
        if general_entries.is_empty() && backpacks.is_empty() {
            println!("No entries found");
            return Ok(());
        }
        
        if json {
            let mut result = serde_json::Map::new();
            result.insert("general".to_string(), serde_json::to_value(&general_entries)?);
            
            for backpack in backpacks {
                let backpack_entries = storage.list_entries(Some(&backpack.name))?;
                result.insert(backpack.name, serde_json::to_value(&backpack_entries)?);
            }
            
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            if !general_entries.is_empty() {
                println!("General entries:");
                for entry in general_entries {
                    println!("  {} - {}", entry.id, entry.title);
                }
            }
            
            for backpack in backpacks {
                let backpack_entries = storage.list_entries(Some(&backpack.name))?;
                if !backpack_entries.is_empty() {
                    println!("\nEntries in backpack '{}':", backpack.name);
                    for entry in backpack_entries {
                        println!("  {} - {}", entry.id, entry.title);
                    }
                }
            }
        }
    } else {
        // List only general entries
        let entries = storage.list_entries(None)?;
        if entries.is_empty() {
            println!("No entries found");
            return Ok(());
        }
        
        if json {
            println!("{}", serde_json::to_string_pretty(&entries)?);
        } else {
            println!("General entries:");
            for entry in entries {
                println!("  {} - {}", entry.id, entry.title);
            }
        }
    }
    
    Ok(())
}

/// Remove an entry from pocket storage
pub fn remove_command(
    id: String,
    force: bool,
    backpack: Option<String>,
) -> Result<()> {
    let storage = StorageManager::new()?;
    
    // Load the entry to show what will be removed
    let (entry, content) = storage.load_entry(&id, backpack.as_deref())
        .with_context(|| format!("Entry with ID '{}' not found", id))?;
    
    // Show the entry and confirm removal
    println!("Entry: {} - {}", entry.id, entry.title);
    println!("Content preview: {}", utils::truncate_string(&content, 100));
    
    if !force {
        let confirmed = utils::confirm("Are you sure you want to remove this entry?", false)?;
        if !confirmed {
            println!("Operation cancelled");
            return Ok(());
        }
    }
    
    // Remove the entry
    storage.remove_entry(&id, backpack.as_deref())?;
    println!("Entry removed successfully");
    
    Ok(())
}

/// Create a new backpack
pub fn create_backpack_command(
    name: String,
    description: Option<String>,
) -> Result<()> {
    let storage = StorageManager::new()?;
    
    // Create the backpack
    let backpack = Backpack::new(name.clone(), description);
    storage.create_backpack(&backpack)?;
    
    println!("Backpack '{}' created successfully", name);
    
    Ok(())
}

/// Search for entries
pub fn search_command(
    query: String,
    limit: usize,
    backpack: Option<String>,
    exact: bool,
) -> Result<()> {
    let storage = StorageManager::new()?;
    let search_engine = SearchEngine::new(storage);
    
    // Load config to get search algorithm
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;
    
    let algorithm = if exact {
        crate::models::SearchAlgorithm::Literal
    } else {
        config.search.algorithm
    };
    
    // Search for entries
    let results = search_engine.search(&query, limit, backpack.as_deref(), algorithm)?;
    
    if results.is_empty() {
        println!("No matching entries found");
        return Ok(());
    }
    
    // Display results
    println!("Search results for '{}':", query);
    for (i, result) in results.iter().enumerate() {
        println!("\n{}. {} - {} (score: {:.2})", 
            i + 1, 
            result.entry.id, 
            result.entry.title,
            result.score
        );
        
        // Show a preview of the content with highlighting
        let preview = search_engine.get_highlighted_content(&result.content, &query, 100);
        println!("   {}", preview);
    }
    
    Ok(())
}

/// Insert an entry into a file
pub fn insert_command(
    id: Option<String>,
    file: Option<String>,
    top: bool,
    no_confirm: bool,
    delimiter: Option<String>,
) -> Result<()> {
    let storage = StorageManager::new()?;
    let search_engine = SearchEngine::new(storage.clone());
    
    // If no ID is provided, search for an entry
    let (entry, content) = if let Some(entry_id) = id {
        storage.load_entry(&entry_id, None)?
    } else {
        // Prompt for a search query
        let query = utils::input::<String>("Enter search query:", None)?;
        
        // Search for entries
        let config = storage.load_config()?;
        let results = search_engine.search(&query, 5, None, config.search.algorithm)?;
        
        if results.is_empty() {
            return Err(anyhow!("No matching entries found"));
        }
        
        // If top flag is set, use the top result
        if top {
            (results[0].entry.clone(), results[0].content.clone())
        } else {
            // Otherwise, prompt the user to select an entry
            let options: Vec<String> = results.iter()
                .map(|r| format!("{} - {}", r.entry.id, r.entry.title))
                .collect();
            
            let selected = utils::select("Select an entry:", &options)?;
            (results[selected].entry.clone(), results[selected].content.clone())
        }
    };
    
    // Determine the target file
    let target_file = if let Some(file_path) = file {
        file_path
    } else {
        // Prompt for a file path
        utils::input::<String>("Enter target file path:", None)?
    };
    
    // Read the target file if it exists
    let path = Path::new(&target_file);
    let existing_content = if path.exists() {
        utils::read_file_content(path)?
    } else {
        String::new()
    };
    
    // Prepare the content to insert with delimiters
    let delim = delimiter.unwrap_or_else(|| "---".to_string());
    let content_with_delimiters = format!(
        "\n{} BEGIN POCKET ENTRY: {} - {} {}\n{}\n{} END POCKET ENTRY {}\n",
        delim, entry.id, entry.title, delim, content, delim, delim
    );
    
    // Show a preview
    println!("Will insert the following content into '{}':", target_file);
    println!("{}", utils::truncate_string(&content_with_delimiters, 200));
    
    // Confirm unless no_confirm is set
    if !no_confirm {
        let confirmed = utils::confirm("Proceed with insertion?", true)?;
        if !confirmed {
            println!("Operation cancelled");
            return Ok(());
        }
    }
    
    // Write the content to the file
    let new_content = format!("{}{}", existing_content, content_with_delimiters);
    std::fs::write(path, new_content)?;
    
    println!("Content inserted successfully into '{}'", target_file);
    
    Ok(())
}

/// Display help information
pub fn help_command(
    command: Option<String>,
    extensions: bool,
) -> Result<()> {
    if extensions {
        println!("No extensions installed yet");
        return Ok(());
    }
    
    // The main help display is now handled in src/main.rs with print_custom_help()
    // This function now only handles specific command help
    
    if let Some(cmd) = command {
        match cmd.as_str() {
            "add" => {
                println!("pocket add [FILE] [OPTIONS]");
                println!("Add content to your pocket storage");
                println!("\nOptions:");
                println!("  -m, --message <TEXT>   Specify text directly");
                println!("  -e, --editor           Open the default editor");
                println!("  -b, --backpack <NAME>  Add directly to a specific backpack");
            }
            "list" => {
                println!("pocket list [OPTIONS]");
                println!("Display all pocket entries");
                println!("\nOptions:");
                println!("  --include-backpacks    Include entries from all backpacks");
                println!("  --backpack <NAME>      Show entries from a specific backpack");
                println!("  --json                 Output in JSON format for scripting");
            }
            "remove" => {
                println!("pocket remove <ID> [OPTIONS]");
                println!("Remove an entry from storage");
                println!("\nOptions:");
                println!("  --force                Skip confirmation prompt");
                println!("  --backpack <NAME>      Specify which backpack to remove from");
            }
            "create" => {
                println!("pocket create backpack <NAME> [OPTIONS]");
                println!("Create a new backpack for organizing entries");
                println!("\nOptions:");
                println!("  --description <TEXT>   Add a description for the backpack");
            }
            "search" => {
                println!("pocket search <QUERY> [OPTIONS]");
                println!("Find entries using semantic similarity");
                println!("\nOptions:");
                println!("  --limit <N>            Limit the number of results (default: 5)");
                println!("  --backpack <NAME>      Search only within a specific backpack");
                println!("  --exact                Use exact text matching instead of semantic search");
            }
            "insert" => {
                println!("pocket insert [ID] [FILE] [OPTIONS]");
                println!("Insert an entry into a file");
                println!("\nOptions:");
                println!("  --top                  Use the top search result");
                println!("  --no-confirm           Skip confirmation");
                println!("  --delimiter <TEXT>     Custom delimiter for inserted content");
            }
            "reload" => {
                println!("pocket reload");
                println!("Reload all extensions");
            }
            "new-repo" => {
                println!("pocket new-repo [PATH] [OPTIONS]");
                println!("Create a new Pocket VCS repository");
                println!("\nOptions:");
                println!("  --template <TEMPLATE>  Initialize with a template");
                println!("  --no-default           Don't create default files");
            }
            "status" => {
                println!("pocket status [OPTIONS]");
                println!("Show repository status");
                println!("\nOptions:");
                println!("  -v, --verbose          Show verbose output");
            }
            "pile" => {
                println!("pocket pile [FILES...] [OPTIONS]");
                println!("Add files to the pile (staging area)");
                println!("\nOptions:");
                println!("  -a, --all              Add all changes");
                println!("  --pattern <PATTERN>    Add files matching pattern");
            }
            "unpile" => {
                println!("pocket unpile [FILES...] [OPTIONS]");
                println!("Remove files from the pile (staging area)");
                println!("\nOptions:");
                println!("  -a, --all              Remove all files");
            }
            "shove" => {
                println!("pocket shove [OPTIONS]");
                println!("Create a shove (commit)");
                println!("\nOptions:");
                println!("  -m, --message <MESSAGE> Commit message");
                println!("  -e, --editor            Open editor for message");
            }
            "log" => {
                println!("pocket log [OPTIONS]");
                println!("Show shove history");
                println!("\nOptions:");
                println!("  -g, --graph            Show graph");
                println!("  --limit <N>            Limit number of entries");
                println!("  --timeline <NAME>      Show history for specific timeline");
            }
            "timeline" => {
                println!("pocket timeline <SUBCOMMAND>");
                println!("Manage timelines (branches)");
                println!("\nSubcommands:");
                println!("  new <NAME>             Create a new timeline");
                println!("  switch <NAME>          Switch to a timeline");
                println!("  list                   List all timelines");
                println!("\nOptions for 'new':");
                println!("  --based-on <SHOVE_ID>  Base the timeline on a specific shove");
            }
            "merge" => {
                println!("pocket merge <NAME> [OPTIONS]");
                println!("Merge a timeline into the current one");
                println!("\nOptions:");
                println!("  --strategy <STRATEGY>  Merge strategy");
            }
            "remote" => {
                println!("pocket remote <SUBCOMMAND>");
                println!("Manage remote repositories");
                println!("\nSubcommands:");
                println!("  add <NAME> <URL>       Add a remote repository");
                println!("  remove <NAME>          Remove a remote repository");
                println!("  list                   List remote repositories");
            }
            "fish" => {
                println!("pocket fish [REMOTE]");
                println!("Fetch from a remote repository");
            }
            "push" => {
                println!("pocket push [REMOTE] [TIMELINE]");
                println!("Push to a remote repository");
            }
            _ => {
                println!("Unknown command: {}", cmd);
                println!("Run 'pocket help' for a list of available commands");
            }
        }
    } else {
        // This should not be reached as main.rs handles the case with no command
        // But just in case, we'll call the custom help
        crate::print_custom_help();
    }
    
    Ok(())
}

/// Execute a workflow
pub fn lint_command(workflow: Option<String>) -> Result<()> {
    let storage = StorageManager::new()?;
    
    if let Some(workflow_str) = workflow {
        // Check if this is a workflow name (without .pocket extension)
        let workflow_path = if !workflow_str.ends_with(".pocket") {
            // Try to find the workflow in the workflows directory
            let mut path = storage.get_workflows_dir()?;
            path.push(format!("{}.pocket", workflow_str));
            path
        } else {
            // Use the provided path directly
            Path::new(&workflow_str).to_path_buf()
        };

        // Check if the workflow file exists
        if workflow_path.exists() {
            println!("Executing workflow from: {}", workflow_path.display());
            
            let content = utils::read_file_content(&workflow_path)?;
            let lines: Vec<String> = content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            // Process each line as a command chain
            for line in lines {
                // Skip comments
                if line.starts_with('#') {
                    continue;
                }
                
                // Execute the command chain
                execute_command_chain(&line)?;
            }
            
            return Ok(());
        } else if !workflow_str.ends_with(".pocket") {
            return Err(anyhow!("Workflow '{}' not found in workflows directory", workflow_str));
        }
        
        // If it's a .pocket file that doesn't exist in the workflows directory,
        // try to execute it as a direct command chain
        execute_command_chain(&workflow_str)?;
    } else {
        // List available workflows from the workflows directory
        let workflows_dir = storage.get_workflows_dir()?;
        if !workflows_dir.exists() {
            println!("No workflows directory found");
            return Ok(());
        }

        let mut found_workflows = false;
        println!("Available workflows:");
        
        // List .pocket files
        if let Ok(entries) = std::fs::read_dir(&workflows_dir) {
            for entry in entries.filter_map(Result::ok) {
                if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".pocket") {
                        found_workflows = true;
                        // Show the workflow name without the .pocket extension
                        let display_name = name.trim_end_matches(".pocket");
                        
                        // Read the first line of the file for description
                        if let Ok(content) = utils::read_file_content(&entry.path()) {
                            let description = content
                                .lines()
                                .find(|line| line.starts_with('#'))
                                .unwrap_or("")
                                .trim_start_matches('#')
                                .trim();
                            
                            println!("  {} - {}", display_name, description);
                        } else {
                            println!("  {}", display_name);
                        }
                    }
                }
            }
        }

        // List saved workflows (legacy format)
        let saved_workflows = storage.list_workflows()?;
        if !saved_workflows.is_empty() {
            found_workflows = true;
            if !saved_workflows.is_empty() {
                println!("\nSaved workflows (legacy format):");
                for workflow in saved_workflows {
                    println!("  {} (created: {})", workflow.name, workflow.created_at);
                    for cmd in workflow.commands {
                        println!("    > {} {}", cmd.command, cmd.args.join(" "));
                    }
                }
            }
        }

        if !found_workflows {
            println!("No workflows found");
        }
    }
    
    Ok(())
}

/// Execute a command chain from a string
fn execute_command_chain(chain: &str) -> Result<()> {
    let storage = StorageManager::new()?;
    
    // Parse the workflow string
    let parts: Vec<&str> = chain.split('>').map(str::trim).filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty workflow"));
    }
    
    // Check if this is a save operation
    if parts.len() >= 2 {
        let last_parts: Vec<&str> = parts[parts.len() - 1].split_whitespace().collect();
        let is_save = parts[parts.len() - 2] == "save" && !last_parts.is_empty();
        
        if is_save {
            // Parse commands (excluding 'save' and name)
            let commands: Result<Vec<WorkflowCommand>> = parts[..parts.len() - 2]
                .iter()
                .filter(|cmd| !cmd.trim().is_empty())
                .map(|cmd| {
                    println!("Parsing command: {}", cmd);
                    WorkflowCommand::parse(cmd)
                })
                .collect();
            
            let commands = commands?;
            println!("Parsed {} commands", commands.len());
            
            // Create and save the workflow
            let workflow = Workflow::new(last_parts[0].to_string(), commands);
            storage.save_workflow(&workflow)?;
            
            println!("Workflow '{}' saved successfully", last_parts[0]);
            return Ok(());
        }
    }
    
    // Execute the workflow immediately
    let commands: Result<Vec<WorkflowCommand>> = parts
        .iter()
        .filter(|cmd| !cmd.trim().is_empty())
        .map(|cmd| WorkflowCommand::parse(cmd))
        .collect();
    
    let commands = commands?;
    
    // Execute each command
    for cmd in commands {
        match cmd.command.as_str() {
            "search" => {
                search_command(
                    cmd.args.join(" "),
                    5,  // default limit
                    None,
                    false,
                )?;
            }
            "insert" => {
                if cmd.args.is_empty() {
                    return Err(anyhow!("Insert command requires a template ID"));
                }
                
                let template_id = cmd.args[0].clone();
                let mut file = None;
                let mut no_confirm = false;
                let mut backpack = None;
                
                // Parse arguments
                let mut i = 1;
                while i < cmd.args.len() {
                    match cmd.args[i].as_str() {
                        "--file" | "-f" => {
                            if i + 1 < cmd.args.len() {
                                file = Some(cmd.args[i + 1].clone());
                                i += 2;
                            } else {
                                return Err(anyhow!("--file requires a file path"));
                            }
                        }
                        "--no-confirm" => {
                            no_confirm = true;
                            i += 1;
                        }
                        "--backpack" | "-b" => {
                            if i + 1 < cmd.args.len() {
                                backpack = Some(cmd.args[i + 1].clone());
                                i += 2;
                            } else {
                                return Err(anyhow!("--backpack requires a backpack name"));
                            }
                        }
                        _ => {
                            i += 1;
                        }
                    }
                }
                
                // Load the entry from the specified backpack
                let storage = StorageManager::new()?;
                let (entry, content) = storage.load_entry(&template_id, backpack.as_deref())?;
                
                // Determine the target file
                let target_file = if let Some(file_path) = file {
                    file_path
                } else {
                    // Prompt for a file path
                    utils::input::<String>("Enter target file path:", None)?
                };
                
                // Read the target file if it exists
                let path = Path::new(&target_file);
                let existing_content = if path.exists() {
                    utils::read_file_content(path)?
                } else {
                    String::new()
                };
                
                // Prepare the content to insert with delimiters
                let delim = "---".to_string();
                let content_with_delimiters = format!(
                    "\n{} BEGIN POCKET ENTRY: {} - {} {}\n{}\n{} END POCKET ENTRY {}\n",
                    delim, entry.id, entry.title, delim, content, delim, delim
                );
                
                // Show a preview
                println!("Will insert the following content into '{}':", target_file);
                println!("{}", utils::truncate_string(&content_with_delimiters, 200));
                
                // Confirm unless no_confirm is set
                if !no_confirm {
                    let confirmed = utils::confirm("Proceed with insertion?", true)?;
                    if !confirmed {
                        println!("Operation cancelled");
                        return Ok(());
                    }
                }
                
                // Write the content to the file
                let new_content = format!("{}{}", existing_content, content_with_delimiters);
                std::fs::write(path, new_content)?;
                
                println!("Content inserted successfully into '{}'", target_file);
            }
            "execute" => {
                if cmd.args.is_empty() {
                    return Err(anyhow!("Execute command requires an ID or file path"));
                }
                
                if cmd.args[0] == "-b" || cmd.args[0] == "--backpack" {
                    if cmd.args.len() < 3 {
                        return Err(anyhow!("Execute command with backpack requires an ID"));
                    }
                    
                    let script_id = cmd.args[2].clone();
                    let backpack = Some(cmd.args[1].clone());
                    let script_args = if cmd.args.len() > 3 {
                        cmd.args[3..].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    execute_command(Some(script_id), None, backpack, true, script_args)?;
                } else if cmd.args[0] == "-f" || cmd.args[0] == "--file" {
                    if cmd.args.len() < 2 {
                        return Err(anyhow!("Execute command with file requires a file path"));
                    }
                    
                    let file_path = cmd.args[1].clone();
                    let script_args = if cmd.args.len() > 2 {
                        cmd.args[2..].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    execute_command(None, Some(file_path), None, true, script_args)?;
                } else {
                    // Assume the first argument is an ID or file path
                    let first_arg = cmd.args[0].clone();
                    let script_args = if cmd.args.len() > 1 {
                        cmd.args[1..].to_vec()
                    } else {
                        Vec::new()
                    };
                    
                    if Path::new(&first_arg).exists() {
                        execute_command(None, Some(first_arg), None, true, script_args)?;
                    } else {
                        execute_command(Some(first_arg), None, None, true, script_args)?;
                    }
                }
            }
            "save" => {
                // Skip save command in direct execution
                continue;
            }
            _ => return Err(anyhow!("Unknown command: {}", cmd.command)),
        }
    }
    
    println!("SUCCESS");
    Ok(())
}

/// Delete a saved workflow
pub fn delete_workflow_command(name: String) -> Result<()> {
    let storage = StorageManager::new()?;
    
    // Try to delete the workflow
    storage.delete_workflow(&name)?;
    println!("Workflow '{}' deleted successfully", name);
    
    Ok(())
}

/// Edit an existing entry
pub fn edit_command(
    id: String,
    backpack: Option<String>,
) -> Result<String> {
    let storage = StorageManager::new()?;
    
    // Load the entry
    let (entry, content) = storage.load_entry(&id, backpack.as_deref())?;
    
    // Open the editor
    let updated_content = utils::edit_entry(&id, &content, entry.content_type.clone())?;
    
    // Check if the content actually changed
    if content == updated_content {
        println!("No changes made to the entry.");
        return Ok(id);
    }
    
    // Create a new entry with updated content
    let title = updated_content.lines().next().unwrap_or("").to_string();
    let updated_entry = Entry {
        id: entry.id.clone(),
        title,
        created_at: entry.created_at,
        updated_at: chrono::Utc::now(),
        source: entry.source,
        tags: entry.tags,
        content_type: utils::detect_content_type(None, Some(&updated_content)),
    };
    
    // Save the updated entry
    storage.save_entry(&updated_entry, &updated_content, backpack.as_deref())?;
    
    println!("Entry {} updated successfully.", id.cyan());
    Ok(id)
}

/// Execute a script
pub fn execute_command(
    id: Option<String>,
    file: Option<String>,
    backpack: Option<String>,
    no_confirm: bool,
    args: Vec<String>,
) -> Result<()> {
    let storage = StorageManager::new()?;
    
    // Function to execute a script with the given content and title
    let execute_script = |content: &str, title: &str| -> Result<()> {
        // Security warning for script execution
        if !no_confirm {
            println!("{}", "⚠️  Warning: Script execution can be dangerous! ⚠️".yellow().bold());
            println!("You're about to execute: {}", title.cyan());
            let args_str = if args.is_empty() { "[none]".to_string() } else { args.join(" ") };
            println!("Arguments: {}", args_str);
            
            if !Confirm::new()
                .with_prompt("Do you want to proceed?")
                .default(false)
                .interact()?
            {
                return Ok(());
            }
        }
        
        println!("Executing script: {}", title);
        
        // Determine shell command based on OS
        let (shell, flag) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };
        
        // Build the command
        let mut cmd = Command::new(shell);
        
        if cfg!(target_os = "windows") {
            // On Windows, join arguments into the script content
            let full_command = if args.is_empty() {
                content.to_string()
            } else {
                format!("{} {}", content, args.join(" "))
            };
            cmd.arg(flag).arg(full_command);
        } else {
            // On Unix, pass args as separate arguments
            cmd.arg(flag).arg(content);
            
            // Add arguments as separate args
            for arg in &args {
                cmd.arg(arg);
            }
        }
        
        // Execute the command
        let output = match cmd.output() {
            Ok(output) => output,
            Err(e) => return Err(anyhow!("Failed to execute script: {} ({})", title, e)),
        };
        
        // Print output
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr).red());
        }
        
        // Check exit status and report
        if !output.status.success() {
            println!("{} Script exited with status: {}", "Error:".red().bold(), 
                output.status.code().map_or("unknown".to_string(), |c| c.to_string()).red());
        }
        
        Ok(())
    };
    
    if let Some(file_path) = file {
        // Execute script from file
        let path = Path::new(&file_path);
        if !path.exists() {
            return Err(anyhow!("Script file not found: {}", file_path));
        }
        
        // Read the file content
        let content = utils::read_file_content(path)?;
        
        // Check if the file is executable
        #[cfg(unix)]
        let (was_executable, made_executable) = ensure_executable(path)?;
        
        // Execute the script
        let result = execute_script(&content, &file_path);
        
        // Restore original permissions if we changed them
        #[cfg(unix)]
        if made_executable && !was_executable {
            restore_permissions(path, was_executable)?;
        }
        
        // Handle result after permissions are restored
        result?;
        
        // Ask if the user wants to add this script to the scripts backpack
        if !no_confirm && Confirm::new()
            .with_prompt("Would you like to add this to executable scripts?")
            .default(false)
            .interact()?
        {
            // Create scripts backpack if it doesn't exist
            let scripts_backpack = Backpack::new("scripts".to_string(), Some("Executable scripts".to_string()));
            if let Err(e) = storage.create_backpack(&scripts_backpack) {
                // Ignore error if backpack already exists
                if !e.to_string().contains("already exists") {
                    return Err(e);
                }
            }
            
            // Add the script to the scripts backpack
            let title = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("script")
                .to_string();
            
            let mut entry = Entry::new(
                title,
                ContentType::Script,
                Some(file_path),
                vec!["executable".to_string()]
            );
            
            // Add example usage as a custom metadata field
            if !args.is_empty() {
                entry.tags.push(format!("args:{}", args.join(" ")));
            }
            
            storage.save_entry(&entry, &content, Some("scripts"))?;
            
            println!("Script added to scripts backpack with ID: {}", entry.id);
        }
        
        return Ok(());
    } else if let Some(id) = id {
        // Execute script from storage
        let backpack_name = backpack.as_deref().unwrap_or("scripts");
        
        // Load the script
        let (entry, content) = storage.load_entry(&id, Some(backpack_name))
            .with_context(|| format!("Failed to load script with ID: {} from backpack: {}", id, backpack_name))?;
        
        // Execute the script
        execute_script(&content, &entry.title)?;
        
        return Ok(());
    }
    
    return Err(anyhow!("No script provided. Use --file or provide an ID"));
}

/// Check if a file is executable and make it executable if needed
#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(bool, bool)> {
    use std::os::unix::fs::PermissionsExt;
    
    // Get current permissions
    let metadata = fs::metadata(path)?;
    let mut perms = metadata.permissions();
    
    // Check if file is already executable by the owner
    let is_executable = perms.mode() & 0o100 != 0;
    
    // If not executable, make it executable
    if !is_executable {
        perms.set_mode(perms.mode() | 0o100); // Add owner executable bit
        fs::set_permissions(path, perms)?;
        // Return (was_executable, made_executable)
        Ok((false, true))
    } else {
        // Already executable, no changes made
        Ok((true, false))
    }
}

/// Restore original permissions
#[cfg(unix)]
fn restore_permissions(path: &Path, was_executable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    if !was_executable {
        let metadata = fs::metadata(path)?;
        let mut perms = metadata.permissions();
        // Remove executable bit
        perms.set_mode(perms.mode() & !0o100);
        fs::set_permissions(path, perms)?;
    }
    
    Ok(())
} 