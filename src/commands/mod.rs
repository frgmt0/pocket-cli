use crate::models::{Entry, Backpack, ContentType, Workflow, WorkflowCommand};
use crate::storage::StorageManager;
use crate::search::SearchEngine;
use crate::utils;
use anyhow::{Result, anyhow, Context};
use std::path::{Path, PathBuf};
use owo_colors::OwoColorize;
use std::process::Command;
use dialoguer::Confirm;
use std::fs;
use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;

// Export blend module
pub mod blend;

/// Add content to pocket storage
pub fn add_command(
    file: Option<String>,
    message: Option<String>,
    editor: bool,
    backpack: Option<String>,
    clipboard: Option<bool>,
    summarize: Option<String>,
) -> Result<String> {
    let storage = StorageManager::new()?;
    
    // Determine the content source
    let (content, file_path) = if let Some(true) = clipboard {
        // Read from clipboard
        (utils::read_clipboard()?, None)
    } else if let Some(ref file_path) = file {
        // Read from file
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(anyhow!("File not found: {}", file_path));
        }
        (fs::read_to_string(path)?, Some(file_path.clone()))
    } else if let Some(msg) = message {
        // Use provided message
        (msg, None)
    } else if editor {
        // Open editor
        (utils::open_editor(None)?, None)
    } else {
        // Read from stdin
        (utils::read_stdin_content()?, None)
    };
    
    if content.trim().is_empty() {
        return Err(anyhow!("Empty content"));
    }
    
    // Detect content type
    let content_type = utils::detect_content_type(file_path.as_deref().map(Path::new), Some(&content));
    
    // Create title from first line
    let title = utils::get_title_from_content(&content);
    
    // Create entry
    let mut entry = Entry::new(title, content_type, file_path, vec![]);
    
    // Add summary if provided or generate one
    if let Some(summary_text) = summarize {
        let summary = utils::SummaryMetadata::new(summary_text, false);
        entry.add_metadata("summary", &summary.to_json());
    } else if content.len() > 100 {
        // Auto-generate summary for longer content
        let summary_text = utils::summarize_text(&content)?;
        let summary = utils::SummaryMetadata::new(summary_text, true);
        entry.add_metadata("summary", &summary.to_json());
    }
    
    // Save entry
    storage.save_entry(&entry, &content, backpack.as_deref())?;
    
    Ok(entry.id)
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
    package: bool,
) -> Result<()> {
    if package {
        return search_packages(&query, limit);
    }
    
    let storage = StorageManager::new()?;
    let search_engine = SearchEngine::new(storage.clone());
    
    // Load config to get search algorithm
    let config = storage.load_config()?;
    
    let algorithm = if exact {
        crate::models::SearchAlgorithm::Literal
    } else {
        config.search.algorithm
    };
    
    // Search for entries (backpack is optional, defaults to searching all backpacks)
    let results = search_engine.search(&query, limit, backpack.as_deref(), algorithm)?;
    
    if results.is_empty() {
        println!("No matching entries found for '{}'", query.cyan());
        return Ok(());
    }
    
    // Display results
    println!("\n🔍 Search results for '{}':", query.cyan().bold());
    println!("─────────────────────────────────────────────────────");
    
    for (i, result) in results.iter().enumerate() {
        // Display result header with backpack info if available
        let backpack_info = match &result.backpack {
            Some(name) => format!(" [in backpack: {}]", name.green()),
            None => "".to_string()
        };
        
        println!("\n{}. {} - {}{} (score: {:.2})", 
            (i + 1).to_string().bold().yellow(), 
            result.entry.id.bright_blue(), 
            result.entry.title.bold(),
            backpack_info,
            result.score
        );
        
        // Show tags if present
        if !result.entry.tags.is_empty() {
            let tags = result.entry.tags.iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join(" ");
            println!("   Tags: {}", tags.cyan());
        }
        
        // Show summary if available
        if let Some(summary_json) = result.entry.get_metadata("summary") {
            if let Ok(summary) = crate::utils::SummaryMetadata::from_json(summary_json) {
                println!("   Summary: {}", summary.summary.bright_white());
            }
        }
        
        // Display highlights
        if !result.highlights.is_empty() {
            println!("   Matching context:");
            for (h_idx, highlight) in result.highlights.iter().enumerate() {
                if h_idx > 0 {
                    println!("   ─────────────");
                }
                println!("   {}", highlight);
            }
        }
    }
    
    // Show usage hint
    println!("\n💡 Tip: Use 'pocket show <ID>' to view entire entry content");
    
    Ok(())
}

/// Search for packages that match the description
fn search_packages(query: &str, limit: usize) -> Result<()> {
    println!("🔍 Searching for packages matching: {}", query.cyan().bold());
    
    // Detect programming language from current directory
    let (language, _files) = detect_language_from_directory(".")?;
    
    println!("📦 Detected language: {}", language.green().bold());
    
    // Search for packages based on language
    let results = search_language_packages(&language, query, limit)?;
    
    if results.is_empty() {
        println!("No matching packages found for '{}'", query.cyan());
        return Ok(());
    }
    
    // Display results
    println!("\n📦 Package results for '{}':", query.cyan().bold());
    println!("─────────────────────────────────────────────────────");
    
    for (i, package) in results.iter().enumerate() {
        println!("\n{}. {} - {}", 
            (i + 1).to_string().bold().yellow(), 
            package.name.bright_blue().bold(), 
            package.description
        );
        println!("   Version: {}", package.version.cyan());
        
        if let Some(stars) = &package.stars {
            println!("   Stars: {}", stars.yellow());
        }
        
        if let Some(url) = &package.url {
            println!("   URL: {}", url);
        }
        
        println!("   Install: {}", package.install_command.green());
    }
    
    Ok(())
}

/// Package information structure
#[derive(Debug, Clone)]
struct PackageInfo {
    name: String,
    description: String,
    version: String,
    stars: Option<String>,
    url: Option<String>,
    install_command: String,
}

/// Detect programming language from files in directory
fn detect_language_from_directory(dir_path: &str) -> Result<(String, Vec<PathBuf>)> {
    let mut extension_counts: HashMap<String, usize> = HashMap::new();
    let mut found_files: Vec<PathBuf> = Vec::new();
    
    // Files to check for package managers
    let package_files = [
        ("package.json", "javascript"),
        ("Cargo.toml", "rust"),
        ("requirements.txt", "python"),
        ("go.mod", "go"),
        ("pom.xml", "java"),
        ("build.gradle", "java"),
        ("composer.json", "php"),
        ("Gemfile", "ruby"),
    ];
    
    // Check for explicit package manager files first
    for (file, language) in &package_files {
        let path = Path::new(dir_path).join(file);
        if path.exists() {
            found_files.push(path.clone());
            return Ok((language.to_string(), found_files));
        }
    }
    
    // Count file extensions
    for entry in WalkDir::new(dir_path)
        .max_depth(3)  // Don't go too deep
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(extension) = entry.path().extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext = ext_str.to_lowercase();
                *extension_counts.entry(ext).or_insert(0) += 1;
                found_files.push(entry.path().to_path_buf());
            }
        }
    }
    
    // Map common extensions to languages
    let mut language = "unknown";
    let mut max_count = 0;
    
    for (ext, count) in &extension_counts {
        if count > &max_count {
            match ext.as_str() {
                "js" | "jsx" | "ts" | "tsx" => {
                    language = "javascript";
                    max_count = *count;
                },
                "py" => {
                    language = "python";
                    max_count = *count;
                },
                "rs" => {
                    language = "rust";
                    max_count = *count;
                },
                "go" => {
                    language = "go";
                    max_count = *count;
                },
                "java" => {
                    language = "java";
                    max_count = *count;
                },
                "kt" => {
                    language = "kotlin";
                    max_count = *count;
                },
                "php" => {
                    language = "php";
                    max_count = *count;
                },
                "rb" => {
                    language = "ruby";
                    max_count = *count;
                },
                "swift" => {
                    language = "swift";
                    max_count = *count;
                },
                "cs" => {
                    language = "csharp";
                    max_count = *count;
                },
                "c" | "h" => {
                    language = "c";
                    max_count = *count;
                },
                "cpp" | "hpp" => {
                    language = "cpp";
                    max_count = *count;
                },
                _ => {}
            }
        }
    }
    
    Ok((language.to_string(), found_files))
}

/// Search for packages based on the detected language
fn search_language_packages(language: &str, query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    match language {
        "javascript" => search_npm_packages(query, limit),
        "python" => search_python_packages(query, limit),
        "rust" => search_rust_packages(query, limit),
        "go" => search_go_packages(query, limit),
        "java" => search_maven_packages(query, limit),
        "ruby" => search_ruby_packages(query, limit),
        "php" => search_php_packages(query, limit),
        _ => Err(anyhow!("Package search not supported for {}", language)),
    }
}

/// Search for npm packages
fn search_npm_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    let output = match Command::new("npm")
        .args(["search", query, "--json", "--no-description"])
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Error executing npm search command: {}", e);
                return fallback_npm_packages(query, limit);
            }
        };

    if !output.status.success() {
        return fallback_npm_packages(query, limit);
    }

    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing npm search output: {}", e);
            return fallback_npm_packages(query, limit);
        }
    };
    
    let results: Vec<serde_json::Value> = match serde_json::from_str(&output_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing npm search JSON response: {}", e);
            return fallback_npm_packages(query, limit);
        }
    };
    
    let mut packages = Vec::new();
    for (i, package) in results.iter().enumerate() {
        if i >= limit {
            break;
        }
        
        if let (Some(name), Some(description), Some(version)) = (
            package["name"].as_str(),
            package["description"].as_str().or(Some("No description available")),
            package["version"].as_str(),
        ) {
            packages.push(PackageInfo {
                name: name.to_string(),
                description: description.to_string(),
                version: version.to_string(),
                stars: None,
                url: Some(format!("https://www.npmjs.com/package/{}", name)),
                install_command: format!("npm install {}", name),
            });
        }
    }
    
    if packages.is_empty() {
        return fallback_npm_packages(query, limit);
    }
    
    Ok(packages)
}

/// Fallback function to provide static package suggestions for npm/JavaScript
fn fallback_npm_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Define some common npm packages for various categories
    let state_management = vec![
        PackageInfo {
            name: "redux".to_string(), 
            description: "Predictable state container for JavaScript apps".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/redux".to_string()),
            install_command: "npm install redux".to_string(),
        },
        PackageInfo {
            name: "mobx".to_string(), 
            description: "Simple, scalable state management".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/mobx".to_string()),
            install_command: "npm install mobx".to_string(),
        },
        PackageInfo {
            name: "zustand".to_string(), 
            description: "Bear necessities for state management in React".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/zustand".to_string()),
            install_command: "npm install zustand".to_string(),
        },
        PackageInfo {
            name: "recoil".to_string(), 
            description: "State management library for React".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/recoil".to_string()),
            install_command: "npm install recoil".to_string(),
        },
        PackageInfo {
            name: "jotai".to_string(), 
            description: "Primitive and flexible state management for React".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/jotai".to_string()),
            install_command: "npm install jotai".to_string(),
        },
    ];
    
    let web = vec![
        PackageInfo {
            name: "express".to_string(), 
            description: "Fast, unopinionated, minimalist web framework for Node.js".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/express".to_string()),
            install_command: "npm install express".to_string(),
        },
        PackageInfo {
            name: "next".to_string(), 
            description: "The React Framework for Production".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/next".to_string()),
            install_command: "npm install next".to_string(),
        },
        PackageInfo {
            name: "fastify".to_string(), 
            description: "Fast and low overhead web framework for Node.js".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/fastify".to_string()),
            install_command: "npm install fastify".to_string(),
        },
        PackageInfo {
            name: "nest".to_string(), 
            description: "A progressive Node.js framework for building efficient and scalable server-side applications".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/@nestjs/core".to_string()),
            install_command: "npm install @nestjs/core".to_string(),
        },
    ];
    
    let utils = vec![
        PackageInfo {
            name: "lodash".to_string(), 
            description: "Lodash modular utilities".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/lodash".to_string()),
            install_command: "npm install lodash".to_string(),
        },
        PackageInfo {
            name: "axios".to_string(), 
            description: "Promise based HTTP client for the browser and node.js".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/axios".to_string()),
            install_command: "npm install axios".to_string(),
        },
        PackageInfo {
            name: "date-fns".to_string(), 
            description: "Modern JavaScript date utility library".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/date-fns".to_string()),
            install_command: "npm install date-fns".to_string(),
        },
        PackageInfo {
            name: "zod".to_string(), 
            description: "TypeScript-first schema validation with static type inference".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://www.npmjs.com/package/zod".to_string()),
            install_command: "npm install zod".to_string(),
        },
    ];
    
    // Check the query to determine which category to return
    let query_lower = query.to_lowercase();
    let selected_packages = if query_lower.contains("state") || query_lower.contains("manage") {
        state_management
    } else if query_lower.contains("web") || query_lower.contains("server") || query_lower.contains("http") {
        web
    } else {
        utils
    };
    
    // Return the appropriate number of packages
    Ok(selected_packages.into_iter().take(limit).collect())
}

/// Search for Python packages
fn search_python_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Use pip search or PyPI API
    // Note: pip search is deprecated, so we'll execute a curl command to query PyPI JSON API
    let output = match Command::new("curl")
        .args(["-s", &format!("https://pypi.org/search/?q={}&format=json", query)])
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Error executing PyPI search command: {}", e);
                return fallback_python_packages(query, limit);
            }
        };

    if !output.status.success() {
        return fallback_python_packages(query, limit);
    }

    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing PyPI search output: {}", e);
            return fallback_python_packages(query, limit);
        }
    };
    
    let results: serde_json::Value = match serde_json::from_str(&output_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing PyPI search JSON response: {}", e);
            return fallback_python_packages(query, limit);
        }
    };
    
    let mut packages = Vec::new();
    
    if let Some(results_array) = results["results"].as_array() {
        for (i, package) in results_array.iter().enumerate() {
            if i >= limit {
                break;
            }
            
            if let (Some(name), Some(version), Some(description)) = (
                package["name"].as_str(),
                package["version"].as_str(),
                package["description"].as_str().or(Some("No description available")),
            ) {
                packages.push(PackageInfo {
                    name: name.to_string(),
                    description: description.to_string(),
                    version: version.to_string(),
                    stars: None,
                    url: Some(format!("https://pypi.org/project/{}", name)),
                    install_command: format!("pip install {}", name),
                });
            }
        }
    }
    
    if packages.is_empty() {
        return fallback_python_packages(query, limit);
    }
    
    Ok(packages)
}

/// Fallback function to provide static package suggestions for Python
fn fallback_python_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Define some common Python packages for various categories
    let state_management = vec![
        PackageInfo {
            name: "pydantic".to_string(), 
            description: "Data validation and settings management using Python type hints".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/pydantic".to_string()),
            install_command: "pip install pydantic".to_string(),
        },
        PackageInfo {
            name: "attrs".to_string(), 
            description: "Classes Without Boilerplate".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/attrs".to_string()),
            install_command: "pip install attrs".to_string(),
        },
        PackageInfo {
            name: "dataclasses".to_string(), 
            description: "A backport of the dataclasses module for Python 3.6".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/dataclasses".to_string()),
            install_command: "pip install dataclasses".to_string(),
        },
        PackageInfo {
            name: "redis".to_string(), 
            description: "Python client for Redis key-value store".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/redis".to_string()),
            install_command: "pip install redis".to_string(),
        },
        PackageInfo {
            name: "sqlalchemy".to_string(), 
            description: "Database Abstraction Library".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/sqlalchemy".to_string()),
            install_command: "pip install sqlalchemy".to_string(),
        },
    ];
    
    let web = vec![
        PackageInfo {
            name: "flask".to_string(), 
            description: "A simple framework for building complex web applications".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/flask".to_string()),
            install_command: "pip install flask".to_string(),
        },
        PackageInfo {
            name: "django".to_string(), 
            description: "A high-level Python Web framework that encourages rapid development".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/django".to_string()),
            install_command: "pip install django".to_string(),
        },
        PackageInfo {
            name: "fastapi".to_string(), 
            description: "FastAPI framework, high performance, easy to learn, fast to code, ready for production".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/fastapi".to_string()),
            install_command: "pip install fastapi".to_string(),
        },
        PackageInfo {
            name: "starlette".to_string(), 
            description: "The little ASGI framework that shines".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/starlette".to_string()),
            install_command: "pip install starlette".to_string(),
        },
    ];
    
    let utils = vec![
        PackageInfo {
            name: "requests".to_string(), 
            description: "Python HTTP for Humans".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/requests".to_string()),
            install_command: "pip install requests".to_string(),
        },
        PackageInfo {
            name: "pandas".to_string(), 
            description: "Powerful data structures for data analysis, time series, and statistics".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/pandas".to_string()),
            install_command: "pip install pandas".to_string(),
        },
        PackageInfo {
            name: "numpy".to_string(), 
            description: "Fundamental package for array computing in Python".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/numpy".to_string()),
            install_command: "pip install numpy".to_string(),
        },
        PackageInfo {
            name: "pytest".to_string(), 
            description: "Simple powerful testing with Python".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://pypi.org/project/pytest".to_string()),
            install_command: "pip install pytest".to_string(),
        },
    ];
    
    // Check the query to determine which category to return
    let query_lower = query.to_lowercase();
    let selected_packages = if query_lower.contains("state") || query_lower.contains("manage") {
        state_management
    } else if query_lower.contains("web") || query_lower.contains("server") || query_lower.contains("http") {
        web
    } else {
        utils
    };
    
    // Return the appropriate number of packages
    Ok(selected_packages.into_iter().take(limit).collect())
}

/// Search for Rust packages (crates)
fn search_rust_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Use a curl command to get data from crates.io API
    let output = match Command::new("curl")
        .args(["-s", &format!("https://crates.io/api/v1/crates?q={}&per_page={}", query, limit)])
        .output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Error executing curl command: {}", e);
                // Fallback to a simpler approach - just provide some common packages related to the search term
                return fallback_rust_packages(query, limit);
            }
        };

    if !output.status.success() {
        return fallback_rust_packages(query, limit);
    }

    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error parsing curl output: {}", e);
            return fallback_rust_packages(query, limit);
        }
    };
    
    let results: serde_json::Value = match serde_json::from_str(&output_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing JSON response: {}", e);
            return fallback_rust_packages(query, limit);
        }
    };
    
    let mut packages = Vec::new();
    
    if let Some(crates) = results["crates"].as_array() {
        for package in crates.iter().take(limit) {
            if let (Some(name), Some(description), Some(version)) = (
                package["name"].as_str(),
                package["description"].as_str().or(Some("No description available")),
                package["max_version"].as_str(),
            ) {
                let downloads = package["downloads"].as_u64().map(|d| format!("{} downloads", d));
                
                packages.push(PackageInfo {
                    name: name.to_string(),
                    description: description.to_string(),
                    version: version.to_string(),
                    stars: downloads,
                    url: Some(format!("https://crates.io/crates/{}", name)),
                    install_command: format!("cargo add {}", name),
                });
            }
        }
    }
    
    if packages.is_empty() {
        return fallback_rust_packages(query, limit);
    }
    
    Ok(packages)
}

/// Fallback function to provide static package suggestions for Rust
fn fallback_rust_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Define some common Rust packages for various categories
    let state_management = vec![
        PackageInfo {
            name: "dashmap".to_string(), 
            description: "Blazing fast concurrent HashMap for Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/dashmap".to_string()),
            install_command: "cargo add dashmap".to_string(),
        },
        PackageInfo {
            name: "im".to_string(), 
            description: "Immutable data structures for Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/im".to_string()),
            install_command: "cargo add im".to_string(),
        },
        PackageInfo {
            name: "arc-swap".to_string(), 
            description: "Atomic swap for Arc, useful for safely sharing state".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/arc-swap".to_string()),
            install_command: "cargo add arc-swap".to_string(),
        },
        PackageInfo {
            name: "once_cell".to_string(), 
            description: "Single assignment cells and lazy values for Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/once_cell".to_string()),
            install_command: "cargo add once_cell".to_string(),
        },
        PackageInfo {
            name: "crossbeam".to_string(), 
            description: "Tools for concurrent programming in Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/crossbeam".to_string()),
            install_command: "cargo add crossbeam".to_string(),
        },
    ];
    
    let web = vec![
        PackageInfo {
            name: "actix-web".to_string(), 
            description: "Fast, pragmatic and flexible web framework for Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/actix-web".to_string()),
            install_command: "cargo add actix-web".to_string(),
        },
        PackageInfo {
            name: "rocket".to_string(), 
            description: "Web framework with a focus on usability, security, and performance".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/rocket".to_string()),
            install_command: "cargo add rocket".to_string(),
        },
        PackageInfo {
            name: "axum".to_string(), 
            description: "Ergonomic and modular web framework built with Tokio, Tower, and Hyper".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/axum".to_string()),
            install_command: "cargo add axum".to_string(),
        },
    ];
    
    let utils = vec![
        PackageInfo {
            name: "serde".to_string(), 
            description: "Serialization framework for Rust".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/serde".to_string()),
            install_command: "cargo add serde".to_string(),
        },
        PackageInfo {
            name: "tokio".to_string(), 
            description: "An event-driven, non-blocking I/O platform for writing asynchronous applications".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/tokio".to_string()),
            install_command: "cargo add tokio".to_string(),
        },
        PackageInfo {
            name: "anyhow".to_string(), 
            description: "Flexible concrete Error type built on std::error::Error".to_string(),
            version: "latest".to_string(),
            stars: Some("Popular".to_string()),
            url: Some("https://crates.io/crates/anyhow".to_string()),
            install_command: "cargo add anyhow".to_string(),
        },
    ];
    
    // Check the query to determine which category to return
    let query_lower = query.to_lowercase();
    let selected_packages = if query_lower.contains("state") || query_lower.contains("manage") {
        state_management
    } else if query_lower.contains("web") || query_lower.contains("server") || query_lower.contains("http") {
        web
    } else {
        utils
    };
    
    // Return the appropriate number of packages
    Ok(selected_packages.into_iter().take(limit).collect())
}

/// Search for Go packages
fn search_go_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Use a curl command to get data from pkg.go.dev via a search engine proxy
    let output = Command::new("curl")
        .args(["-s", &format!("https://pkg.go.dev/search?q={}&limit={}", query, limit)])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to search Go packages"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    
    // Parse the HTML response
    let mut packages = Vec::new();
    
    // Simple regex-based parsing (in a real implementation, use an HTML parser)
    let package_regex = Regex::new(r#"<a class="go-Package-title" href="([^"]+)">([^<]+)</a>.*?<p class="go-Package-synopsis">([^<]+)</p>"#).unwrap();
    
    for cap in package_regex.captures_iter(&output_str).take(limit) {
        let path = cap.get(1).map_or("", |m| m.as_str());
        let name = cap.get(2).map_or("", |m| m.as_str());
        let description = cap.get(3).map_or("", |m| m.as_str());
        
        let full_path = if path.starts_with("/") {
            format!("github.com{}", path)
        } else {
            path.to_string()
        };
        
        packages.push(PackageInfo {
            name: name.to_string(),
            description: description.to_string(),
            version: "latest".to_string(),
            stars: None,
            url: Some(format!("https://pkg.go.dev{}", path)),
            install_command: format!("go get {}", full_path),
        });
    }
    
    Ok(packages)
}

/// Search for Maven packages (Java)
fn search_maven_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    // Use Maven Central's search API
    let output = Command::new("curl")
        .args(["-s", &format!("https://search.maven.org/solrsearch/select?q={}&rows={}&wt=json", query, limit)])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to search Maven packages"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    let results: serde_json::Value = serde_json::from_str(&output_str)?;
    
    let mut packages = Vec::new();
    
    if let Some(docs) = results["response"]["docs"].as_array() {
        for package in docs.iter() {
            if let (Some(group_id), Some(artifact_id), Some(version)) = (
                package["g"].as_str(),
                package["a"].as_str(),
                package["latestVersion"].as_str(),
            ) {
                let name = format!("{}:{}", group_id, artifact_id);
                packages.push(PackageInfo {
                    name,
                    description: package["text"].as_array()
                        .and_then(|a| a.get(0))
                        .and_then(|t| t.as_str())
                        .unwrap_or("No description")
                        .to_string(),
                    version: version.to_string(),
                    stars: None,
                    url: Some(format!("https://search.maven.org/artifact/{}/{}/{}", 
                        group_id, artifact_id, version)),
                    install_command: format!(
                        "<!-- Maven -->\n<dependency>\n  <groupId>{}</groupId>\n  <artifactId>{}</artifactId>\n  <version>{}</version>\n</dependency>\n\n<!-- Gradle -->\nimplementation '{}:{}:{}'", 
                        group_id, artifact_id, version, group_id, artifact_id, version
                    ),
                });
            }
        }
    }
    
    Ok(packages)
}

/// Search for Ruby packages (gems)
fn search_ruby_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    let output = Command::new("gem")
        .args(["search", query, "--remote", "--limit", &limit.to_string()])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to search Ruby gems"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    let lines: Vec<&str> = output_str.lines().collect();
    
    let mut packages = Vec::new();
    let re = Regex::new(r"^([\w-]+) \(([^)]+)\)$").unwrap();
    
    for line in lines {
        if let Some(caps) = re.captures(line) {
            let name = caps.get(1).map_or("", |m| m.as_str());
            let version = caps.get(2).map_or("", |m| m.as_str());
            
            let description = if let Ok(desc_output) = Command::new("gem")
                .args(["info", "--remote", name])
                .output() 
            {
                let desc_str = String::from_utf8_lossy(&desc_output.stdout);
                let desc_re = Regex::new(r"(?m)^    (.+)$").unwrap();
                desc_re.captures(&desc_str)
                    .and_then(|cap| cap.get(1))
                    .map_or("No description", |m| m.as_str())
                    .to_string()
            } else {
                "No description".to_string()
            };

            packages.push(PackageInfo {
                name: name.to_string(),
                description,
                version: version.to_string(),
                stars: None,
                url: Some(format!("https://rubygems.org/gems/{}", name)),
                install_command: format!("gem install {}", name),
            });
        }
        
        if packages.len() >= limit {
            break;
        }
    }
    
    Ok(packages)
}

/// Search for PHP packages (composer)
fn search_php_packages(query: &str, limit: usize) -> Result<Vec<PackageInfo>> {
    let output = Command::new("composer")
        .args(["search", "--format=json", query])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to search PHP packages"));
    }

    let output_str = String::from_utf8(output.stdout)?;
    let results: serde_json::Value = serde_json::from_str(&output_str)?;
    
    let mut packages = Vec::new();
    
    if let Some(packages_obj) = results["packages"].as_object() {
        for (i, (name, package)) in packages_obj.iter().enumerate() {
            if i >= limit {
                break;
            }
            
            if let (Some(description), Some(version)) = (
                package["description"].as_str(),
                package["versions"].as_array().and_then(|v| v.first()).and_then(|v| v.as_str()),
            ) {
                packages.push(PackageInfo {
                    name: name.to_string(),
                    description: description.to_string(),
                    version: version.to_string(),
                    stars: None,
                    url: Some(format!("https://packagist.org/packages/{}", name)),
                    install_command: format!("composer require {}", name),
                });
            }
        }
    }
    
    Ok(packages)
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
    let updated_entry = Entry {
        id: entry.id.clone(),
        title: entry.title.clone(),
        created_at: entry.created_at,
        updated_at: chrono::Utc::now(),
        source: entry.source.clone(),
        tags: entry.tags.clone(),
        content_type: entry.content_type.clone(),
        metadata: entry.metadata.clone(),
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