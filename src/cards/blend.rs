use crate::cards::{Card, CardConfig, CardCommand};
use crate::utils;
use anyhow::{Result, Context, anyhow};
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::process::Command;

/// Card for shell integration via the blend command
pub struct BlendCard {
    /// Name of the card
    name: String,
    
    /// Version of the card
    version: String,
    
    /// Description of the card
    description: String,
    
    /// Configuration for the card
    config: BlendCardConfig,
    
    /// Path to the Pocket data directory (kept for future use)
    _data_dir: PathBuf,
}

/// Configuration for the blend card
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlendCardConfig {
    /// Path to the hook directory
    pub hook_dir: String,
    
    /// Path to the bin directory
    pub bin_dir: String,
}

impl Default for BlendCardConfig {
    fn default() -> Self {
        Self {
            hook_dir: "~/.pocket/hooks".to_string(),
            bin_dir: "~/.pocket/bin".to_string(),
        }
    }
}

impl BlendCard {
    /// Creates a new blend card
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        Self {
            name: "blend".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: "Shell integration and executable hooks".to_string(),
            config: BlendCardConfig::default(),
            _data_dir: data_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Add a shell script as a hook
    pub fn add_hook(&self, script_path: &str, executable: bool) -> Result<()> {
        // Expand the hook directory path
        let hook_dir = utils::expand_path(&self.config.hook_dir)?;
        
        // Create hook directory if it doesn't exist
        if !hook_dir.exists() {
            fs::create_dir_all(&hook_dir)
                .with_context(|| format!("Failed to create hook directory at {}", hook_dir.display()))?;
        }
        
        // Read the script content
        let script_content = fs::read_to_string(script_path)
            .with_context(|| format!("Failed to read script at {}", script_path))?;
        
        // Determine the hook name (filename without extension)
        let script_path = std::path::Path::new(script_path);
        let hook_name = script_path.file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| anyhow!("Invalid script filename"))?;
        
        // Path to the copied hook script
        let hook_script_path = hook_dir.join(format!("{}.sh", hook_name));
        
        // Write the script to the hook directory
        fs::write(&hook_script_path, script_content)
            .with_context(|| format!("Failed to write hook script to {}", hook_script_path.display()))?;
        
        if executable {
            // Make the script executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&hook_script_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&hook_script_path, perms)?;
            }
            
            // Create the bin directory if it doesn't exist
            let bin_dir = utils::expand_path(&self.config.bin_dir)?;
            if !bin_dir.exists() {
                fs::create_dir_all(&bin_dir)
                    .with_context(|| format!("Failed to create bin directory at {}", bin_dir.display()))?;
                
                // Add the bin directory to PATH
                self.add_bin_to_path(&bin_dir)?;
            }
            
            // Create a wrapper script that calls the hook
            let wrapper_path = bin_dir.join(format!("@{}", hook_name));
            let wrapper_content = format!(
                "#!/bin/bash\n\
                # Wrapper for Pocket hook: {}\n\
                exec \"{}\" \"$@\"\n",
                hook_name,
                hook_script_path.display()
            );
            
            fs::write(&wrapper_path, wrapper_content)
                .with_context(|| format!("Failed to write wrapper script to {}", wrapper_path.display()))?;
            
            // Make the wrapper executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&wrapper_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&wrapper_path, perms)?;
            }
            
            println!("Successfully added executable hook '{}' from {}", hook_name, script_path.display());
            println!("You can run it with '@{}' or 'pocket blend run {}'", hook_name, hook_name);
        } else {
            // Add the hook to shell config
            self.add_hook_to_shell_config(hook_name, &hook_script_path)?;
            println!("Successfully added hook '{}' from {}", hook_name, script_path.display());
            println!("Restart your shell or run 'source {}' to apply changes", self.get_shell_config_path()?.display());
        }
        
        Ok(())
    }
    
    /// List all installed hooks
    pub fn list_hooks(&self) -> Result<()> {
        // Expand the hook directory path
        let hook_dir = utils::expand_path(&self.config.hook_dir)?;
        
        if !hook_dir.exists() {
            println!("No hooks installed yet");
            return Ok(());
        }
        
        let mut hooks = Vec::new();
        
        // Read the hook directory
        for entry in fs::read_dir(hook_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("sh") {
                let name = path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                // Check if it's an executable hook
                let bin_dir = utils::expand_path(&self.config.bin_dir)?;
                let wrapper_path = bin_dir.join(format!("@{}", name));
                let is_executable = wrapper_path.exists();
                
                hooks.push((name, path, is_executable));
            }
        }
        
        if hooks.is_empty() {
            println!("No hooks installed yet");
            return Ok(());
        }
        
        println!("Installed hooks:");
        for (name, path, is_executable) in hooks {
            let hook_type = if is_executable {
                "[executable]"
            } else {
                "[shell extension]"
            };
            
            println!("  @{} ({}) {}", name, path.display(), hook_type);
        }
        
        Ok(())
    }
    
    /// Edit a hook
    pub fn edit_hook(&self, hook_name: &str) -> Result<()> {
        // Remove @ prefix if present
        let hook_name = hook_name.trim_start_matches('@');
        
        // Expand the hook directory path
        let hook_dir = utils::expand_path(&self.config.hook_dir)?;
        let hook_path = hook_dir.join(format!("{}.sh", hook_name));
        
        if !hook_path.exists() {
            return Err(anyhow!("Hook '{}' not found", hook_name));
        }
        
        // Get the editor from environment
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        
        // Open the hook script in the editor
        let status = Command::new(&editor)
            .arg(&hook_path)
            .status()
            .with_context(|| format!("Failed to open editor {}", editor))?;
        
        if !status.success() {
            return Err(anyhow!("Editor exited with non-zero status"));
        }
        
        println!("Hook '{}' edited successfully", hook_name);
        Ok(())
    }
    
    /// Run a hook
    pub fn run_hook(&self, hook_name: &str, args: &[String]) -> Result<()> {
        // Remove @ prefix if present
        let hook_name = hook_name.trim_start_matches('@');
        
        // Expand the hook directory path
        let hook_dir = utils::expand_path(&self.config.hook_dir)?;
        let hook_path = hook_dir.join(format!("{}.sh", hook_name));
        
        if !hook_path.exists() {
            return Err(anyhow!("Hook '{}' not found", hook_name));
        }
        
        println!("Running hook '{}'...", hook_name);
        
        // Make sure the script is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }
        
        // Run the hook script with arguments
        let mut command = Command::new(&hook_path);
        if !args.is_empty() {
            command.args(args);
        }
        
        let status = command
            .status()
            .with_context(|| format!("Failed to execute hook '{}'", hook_name))?;
        
        if !status.success() {
            return Err(anyhow!("Hook '{}' exited with non-zero status", hook_name));
        }
        
        Ok(())
    }
    
    /// Get the user's shell config file path
    fn get_shell_config_path(&self) -> Result<PathBuf> {
        // Detect the shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let home = utils::expand_path("~")?;
        
        // Choose the config file based on the shell
        let config_path = if shell.contains("zsh") {
            home.join(".zshrc")
        } else if shell.contains("bash") {
            // Check if .bash_profile exists, otherwise use .bashrc
            let bash_profile = home.join(".bash_profile");
            if bash_profile.exists() {
                bash_profile
            } else {
                home.join(".bashrc")
            }
        } else {
            // Default to .profile
            home.join(".profile")
        };
        
        Ok(config_path)
    }
    
    /// Add hook to shell config
    fn add_hook_to_shell_config(&self, hook_name: &str, hook_path: &PathBuf) -> Result<()> {
        let config_path = self.get_shell_config_path()?;
        
        // Read the current shell config
        let mut config_content = String::new();
        if config_path.exists() {
            let mut file = fs::File::open(&config_path)?;
            file.read_to_string(&mut config_content)?;
        }
        
        // Check if the hook is already in the config
        let source_line = format!("source \"{}\"", hook_path.display());
        if config_content.contains(&source_line) {
            println!("Hook '{}' is already sourced in {}", hook_name, config_path.display());
            return Ok(());
        }
        
        // Add the hook to the shell config
        let mut file = fs::OpenOptions::new()
            
            .append(true)
            .create(true)
            .open(&config_path)?;
        
        writeln!(file, "\n# Pocket CLI hook: {}", hook_name)?;
        writeln!(file, "{}", source_line)?;
        
        println!("Added hook '{}' to {}", hook_name, config_path.display());
        Ok(())
    }
    
    /// Add bin directory to PATH
    fn add_bin_to_path(&self, bin_dir: &PathBuf) -> Result<()> {
        let config_path = self.get_shell_config_path()?;
        
        // Read the current shell config
        let mut config_content = String::new();
        if config_path.exists() {
            let mut file = fs::File::open(&config_path)?;
            file.read_to_string(&mut config_content)?;
        }
        
        // Check if the PATH addition is already in the config
        let path_line = format!("export PATH=\"{}:$PATH\"", bin_dir.display());
        if config_content.contains(&path_line) {
            return Ok(());
        }
        
        // Add the bin directory to PATH
        let mut file = fs::OpenOptions::new()
            
            .append(true)
            .create(true)
            .open(&config_path)?;
        
        writeln!(file, "\n# Pocket hook bin directory")?;
        writeln!(file, "{}", path_line)?;
        
        println!("Added Pocket hook bin directory to your PATH");
        Ok(())
    }
}

impl Card for BlendCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn _description(&self) -> &str {
        &self.description
    }
    
    fn _initialize(&mut self, config: &CardConfig) -> Result<()> {
        // If there are options in the card config, try to parse them
        if let Some(options_value) = config.options.get("blend") {
            if let Ok(options) = serde_json::from_value::<BlendCardConfig>(options_value.clone()) {
                self.config = options;
            }
        }
        
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "add" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing script path"));
                }
                
                let script_path = &args[0];
                
                let mut executable = false;
                
                // Parse optional arguments
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--executable" | "-e" => {
                            executable = true;
                        }
                        _ => { /* Ignore unknown args */ }
                    }
                    i += 1;
                }
                
                self.add_hook(script_path, executable)?;
            }
            "list" => {
                self.list_hooks()?;
            }
            "edit" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing hook name"));
                }
                
                let hook_name = &args[0];
                self.edit_hook(hook_name)?;
            }
            "run" => {
                if args.is_empty() {
                    return Err(anyhow!("Missing hook name"));
                }
                
                let hook_name = &args[0];
                let hook_args = if args.len() > 1 {
                    &args[1..]
                } else {
                    &[]
                };
                
                self.run_hook(hook_name, hook_args)?;
            }
            _ => {
                return Err(anyhow!("Unknown command: {}", command));
            }
        }
        
        Ok(())
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "add".to_string(),
                description: "Add a shell script as a hook".to_string(),
                usage: "add <script_path> [--executable]".to_string(),
            },
            CardCommand {
                name: "list".to_string(),
                description: "List all installed hooks".to_string(),
                usage: "list".to_string(),
            },
            CardCommand {
                name: "edit".to_string(),
                description: "Edit an existing hook".to_string(),
                usage: "edit <hook_name>".to_string(),
            },
            CardCommand {
                name: "run".to_string(),
                description: "Run a hook command directly".to_string(),
                usage: "run <hook_name> [args...]".to_string(),
            },
        ]
    }
    
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
} 