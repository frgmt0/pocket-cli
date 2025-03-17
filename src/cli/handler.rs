use crate::cli::{Cli, Commands, CardOperation, BlendCommands};
use crate::cards::CardManager;
use crate::errors::{PocketError, PocketResult, IntoPocketError};
use crate::logging;
use log::{debug, info, warn, error, LevelFilter};
use std::path::PathBuf;
use colored::Colorize;

/// Handle the CLI command
pub fn handle_command(cli: Cli) -> PocketResult<()> {
    // Set up logging based on verbosity
    let log_level = match cli.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    logging::init(log_level);
    
    debug!("Starting pocket CLI with verbosity level {}", cli.verbose);
    
    // Get the home directory
    let home_dir = std::env::var("HOME")
        .map_err(|_| PocketError::Config("HOME environment variable not set".to_string()))?;
    let data_dir = PathBuf::from(&home_dir).join(".pocket");
    
    // Initialize the card manager
    let card_dir = data_dir.join("cards");
    let mut card_manager = CardManager::new(card_dir.clone());
    card_manager.load_cards()
        .map_err(|e| PocketError::Card(format!("Failed to load cards: {}", e)))?;
    
    // Handle the command
    match cli.command {
        Commands::Add { file, message, editor, backpack, clipboard, summarize } => {
            // Build the arguments for the snippet card
            let mut args = Vec::new();
            
            if let Some(f) = file {
                args.push(format!("--file={}", f));
            }
            
            if let Some(m) = message {
                args.push(format!("--message={}", m));
            }
            
            if editor {
                args.push("--editor".to_string());
            }
            
            if let Some(b) = backpack {
                args.push(format!("--backpack={}", b));
            }
            
            if clipboard {
                args.push("--clipboard".to_string());
            }
            
            if let Some(s) = summarize {
                args.push(format!("--summarize={}", s));
            }
            
            // Execute the command
            card_manager.execute_command("snippet", "add", &args)
                .map_err(|e| PocketError::Card(format!("Failed to add snippet: {}", e)))?;
        },
        
        Commands::List { all, backpack, json, limit } => {
            // Build the arguments for the core card
            let mut args = Vec::new();
            
            if all {
                args.push("--include-backpacks".to_string());
            }
            
            if let Some(b) = backpack {
                args.push("--backpack".to_string());
                args.push(b);
            }
            
            if json {
                args.push("--json".to_string());
            }
            
            args.push("--limit".to_string());
            args.push(limit.to_string());
            
            // Execute the command
            card_manager.execute_command("core", "list", &args)
                .map_err(|e| PocketError::Card(format!("Failed to list entries: {}", e)))?;
        },
        
        Commands::Remove { id, force, backpack } => {
            // Build the arguments for the core card
            let mut args = vec![id];
            
            if force {
                args.push("--force".to_string());
            }
            
            if let Some(b) = backpack {
                args.push("--backpack".to_string());
                args.push(b);
            }
            
            // Execute the command
            card_manager.execute_command("core", "remove", &args)
                .map_err(|e| PocketError::Card(format!("Failed to remove entry: {}", e)))?;
        },
        
        Commands::Create { name, description } => {
            // Build the arguments for the core card
            let mut args = vec![name];
            
            if let Some(d) = description {
                args.push("--description".to_string());
                args.push(d);
            }
            
            // Execute the command
            card_manager.execute_command("core", "create-backpack", &args)
                .map_err(|e| PocketError::Card(format!("Failed to create backpack: {}", e)))?;
        },
        
        Commands::Search { query, limit, backpack, exact, package } => {
            if package {
                // Special case for package search (not yet migrated to card system)
                logging::warning("Package search is not yet migrated to the card system");
                logging::warning("This will be implemented in a future version");
                return Ok(());
            }
            
            // Build the arguments for the core card
            let mut args = vec![query];
            
            args.push("--limit".to_string());
            args.push(limit.to_string());
            
            if let Some(b) = backpack {
                args.push("--backpack".to_string());
                args.push(b);
            }
            
            if exact {
                args.push("--exact".to_string());
            }
            
            // Execute the command
            card_manager.execute_command("core", "search", &args)
                .map_err(|e| PocketError::Card(format!("Failed to search entries: {}", e)))?;
        },
        
        Commands::Insert { id, file, top, no_confirm, delimiter } => {
            if let Some(id) = id {
                if let Some(file_path) = file {
                    // Build the arguments for the core card
                    let mut args = vec![id, file_path];
                    
                    if no_confirm {
                        args.push("--no-confirm".to_string());
                    }
                    
                    if let Some(d) = delimiter {
                        args.push("--delimiter".to_string());
                        args.push(d);
                    }
                    
                    // Execute the command
                    card_manager.execute_command("core", "insert", &args)
                        .map_err(|e| PocketError::Card(format!("Failed to insert entry: {}", e)))?;
                } else {
                    return Err(PocketError::Cli("Missing file path for insert".to_string()));
                }
            } else if top {
                // Handle top entry insertion (not yet fully migrated to card system)
                return Err(PocketError::Cli("Operation not yet supported in the card system".to_string()));
            } else {
                return Err(PocketError::Cli("Missing entry ID for insert".to_string()));
            }
        },
        
        Commands::Reload => {
            logging::info("Reloading all extensions and cards...");
            
            // Re-initialize the card manager
            card_manager = CardManager::new(card_dir.clone());
            card_manager.load_cards()
                .map_err(|e| PocketError::Card(format!("Failed to reload cards: {}", e)))?;
            
            logging::success("Extensions and cards reloaded successfully");
        },
        
        Commands::ShowHelp { command, extensions } => {
            if extensions {
                // Show card commands
                let commands = card_manager.list_commands();
                
                println!("{}", logging::header("Available extensions:"));
                for (card_name, card_commands) in commands {
                    println!("\n{}", logging::title(&card_name));
                    for cmd in card_commands {
                        println!("  {} - {}", logging::key(&cmd.name), cmd.description);
                        println!("    Usage: {}", cmd.usage);
                    }
                }
            } else if let Some(cmd) = command {
                // Show help for a specific command
                // TODO: Implement this with card system
                logging::warning("Command-specific help not yet implemented in the card system");
                logging::warning("This will be improved in a future version");
            } else {
                // Show general help
                print_custom_help();
            }
        },
        
        Commands::Lint { workflow } => {
            // TODO: Migrate to card system
            logging::warning("Lint command not yet migrated to the card system");
            logging::warning("This will be implemented in a future version");
        },
        
        Commands::DeleteWorkflow { name } => {
            // TODO: Migrate to card system
            logging::warning("DeleteWorkflow command not yet migrated to the card system");
            logging::warning("This will be implemented in a future version");
        },
        
        Commands::Version => {
            // Show version information
            println!("Pocket CLI v{}", env!("CARGO_PKG_VERSION"));
            println!("A powerful tool for managing code snippets and shell integrations");
        },
        
        Commands::Edit { id, force, backpack } => {
            // Build the arguments for the core card
            let mut args = vec![id];
            
            if force {
                args.push("--force".to_string());
            }
            
            if let Some(b) = backpack {
                args.push("--backpack".to_string());
                args.push(b);
            }
            
            // TODO: Migrate to card system
            logging::warning("Edit command not yet fully migrated to the card system");
            logging::warning("This will be improved in a future version");
        },
        
        Commands::Execute { name, args } => {
            // TODO: Migrate to card system
            logging::warning("Execute command not yet migrated to the card system");
            logging::warning("This will be implemented in a future version");
        },
        
        Commands::Cards { operation } => {
            match operation {
                Some(CardOperation::List { detail }) => {
                    // List all cards
                    println!("{}", logging::header("Available cards:"));
                    for (name, version, enabled) in card_manager.list_cards() {
                        let status = if enabled {
                            "[Enabled]".green().bold()
                        } else {
                            "[Disabled]".yellow().bold()
                        };
                        
                        println!("{} {} v{}", status, logging::title(&name), version);
                        
                        // List commands for this card
                        if detail {
                            if let Ok(commands) = card_manager.get_card_commands(&name) {
                                for cmd in commands {
                                    println!("  - {}: {}", cmd.name, cmd.description);
                                }
                            }
                        }
                        
                        println!();
                    }
                },
                
                Some(CardOperation::Enable { name }) => {
                    // Enable a card
                    card_manager.enable_card(&name)
                        .map_err(|e| PocketError::Card(format!("Failed to enable card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} enabled", name));
                },
                
                Some(CardOperation::Disable { name }) => {
                    // Disable a card
                    card_manager.disable_card(&name)
                        .map_err(|e| PocketError::Card(format!("Failed to disable card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} disabled", name));
                },
                
                Some(CardOperation::Add { name, url }) => {
                    // Add a new card
                    card_manager.register_card_config(&name, &url)
                        .map_err(|e| PocketError::Card(format!("Failed to add card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} added from {}", name, url));
                },
                
                Some(CardOperation::Remove { name, force }) => {
                    // Remove a card
                    if !force {
                        println!("Are you sure you want to remove card {}? [y/N]", name);
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)
                            .map_err(|e| PocketError::Cli(format!("Failed to read input: {}", e)))?;
                        
                        if !input.trim().eq_ignore_ascii_case("y") {
                            logging::info("Operation cancelled");
                            return Ok(());
                        }
                    }
                    
                    card_manager.remove_card_config(&name)
                        .map_err(|e| PocketError::Card(format!("Failed to remove card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} removed", name));
                },
                
                Some(CardOperation::Build { name, release }) => {
                    // Build a card
                    card_manager.build_card(&name, release)
                        .map_err(|e| PocketError::Card(format!("Failed to build card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} built successfully", name));
                },
                
                Some(CardOperation::Create { name, description }) => {
                    // Create a new card
                    card_manager.create_card(&name, &description)
                        .map_err(|e| PocketError::Card(format!("Failed to create card {}: {}", name, e)))?;
                    
                    logging::success(&format!("Card {} created successfully", name));
                },
                
                None => {
                    // Show help for the cards command
                    println!("{}", logging::header("Card Management:"));
                    println!("  Use the following commands to manage cards:");
                    println!("    pocket cards list       - List all cards");
                    println!("    pocket cards enable     - Enable a card");
                    println!("    pocket cards disable    - Disable a card");
                    println!("    pocket cards add        - Add a new card");
                    println!("    pocket cards remove     - Remove a card");
                    println!("    pocket cards build      - Build a card");
                    println!("    pocket cards create     - Create a new card template");
                    println!("");
                    println!("  For more information, run: pocket help cards");
                }
            }
        },
        
        Commands::Blend { script_file, executable, command } => {
            match command {
                Some(BlendCommands::Edit { hook_name }) => {
                    // Build the arguments for the blend card
                    let args = vec![hook_name];
                    
                    // Execute the command
                    card_manager.execute_command("blend", "edit", &args)
                        .map_err(|e| PocketError::Card(format!("Failed to edit hook: {}", e)))?;
                },
                
                Some(BlendCommands::List) => {
                    // Execute the command
                    card_manager.execute_command("blend", "list", &[])
                        .map_err(|e| PocketError::Card(format!("Failed to list hooks: {}", e)))?;
                },
                
                Some(BlendCommands::Run { hook_name, args }) => {
                    // Build the arguments for the blend card
                    let mut run_args = vec![hook_name];
                    run_args.extend(args.iter().cloned());
                    
                    // Execute the command
                    card_manager.execute_command("blend", "run", &run_args)
                        .map_err(|e| PocketError::Card(format!("Failed to run hook: {}", e)))?;
                },
                
                None => {
                    // Add a script
                    if let Some(script_path) = script_file {
                        let mut args = vec![script_path];
                        
                        if executable {
                            args.push("--executable".to_string());
                        }
                        
                        // Execute the command
                        card_manager.execute_command("blend", "add", &args)
                            .map_err(|e| PocketError::Card(format!("Failed to add hook: {}", e)))?;
                    } else {
                        // Show help for the blend command
                        println!("{}", logging::header("Blend Command:"));
                        println!("  Use the following syntax to blend shell scripts:");
                        println!("    pocket blend <script_file>           - Add a shell extension (sourced at shell startup)");
                        println!("    pocket blend --executable <script>   - Add an executable hook command (run with @name)");
                        println!("");
                        println!("  Other commands:");
                        println!("    pocket blend list                    - List all installed hooks");
                        println!("    pocket blend edit <hook_name>        - Edit an existing hook");
                        println!("    pocket blend run <hook_name> [args]  - Run a hook directly");
                        println!("");
                        println!("  For more information, run: pocket help blend");
                    }
                }
            }
        },
    }
    
    Ok(())
}

/// Print custom help message
fn print_custom_help() {
    println!("{}", logging::header("Pocket CLI Help"));
    println!("A CLI tool for saving, organizing, and retrieving code snippets");
    println!("with integrated version control and shell integration");
    println!("");
    
    println!("{}", logging::header("Core Commands:"));
    println!("  {} - Add content to your pocket storage", logging::key("add"));
    println!("  {} - Display all pocket entries", logging::key("list"));
    println!("  {} - Remove an entry from storage", logging::key("remove"));
    println!("  {} - Create a new backpack for organizing entries", logging::key("create"));
    println!("  {} - Find entries across all backpacks", logging::key("search"));
    println!("  {} - Insert an entry into a file", logging::key("insert"));
    println!("  {} - Reload all extensions", logging::key("reload"));
    println!("  {} - Display help information", logging::key("help"));
    println!("  {} - Lint code before adding", logging::key("lint"));
    println!("  {} - Display version information", logging::key("version"));
    println!("  {} - Edit an existing entry", logging::key("edit"));
    println!("  {} - Execute a script", logging::key("execute"));
    println!("");
    
    println!("{}", logging::header("Extension Commands:"));
    println!("  {} - Manage extensions/cards", logging::key("cards"));
    println!("  {} - Blend shell scripts into your environment", logging::key("blend"));
    println!("");
    
    println!("For more detailed help on a specific command, run:");
    println!("  pocket help <command>");
    println!("");
    
    println!("To see all extensions and their commands, run:");
    println!("  pocket help --extensions");
    println!("");
} 