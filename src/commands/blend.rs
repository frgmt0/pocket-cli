use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{anyhow, Context, Result};
use clap::{Args, Subcommand};
use dirs::home_dir;

#[derive(Debug, Args)]
pub struct BlendArgs {
    #[command(subcommand)]
    pub command: Option<BlendCommands>,

    /// Path to shell script file to blend into shell configuration
    #[arg(global = false, conflicts_with = "command")]
    pub script_file: Option<String>,

    /// Create as an executable hook command (accessible with @name)
    #[arg(long, short)]
    pub executable: bool,
}

#[derive(Debug, Subcommand)]
pub enum BlendCommands {
    /// Edit an existing hook
    Edit {
        /// Name of the hook to edit (with or without @ prefix)
        hook_name: String,
    },
    
    /// List all installed hooks
    List,

    /// Run a hook command directly
    Run {
        /// Name of the hook to run (with or without @ prefix)
        hook_name: String,

        /// Arguments to pass to the hook
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

struct Hook {
    name: String,
    path: PathBuf,
    executable: bool,
}

impl BlendArgs {
    pub fn run(&self) -> Result<()> {
        // Create hooks directory if it doesn't exist
        let hooks_dir = get_hooks_dir()?;
        fs::create_dir_all(&hooks_dir)?;

        match &self.command {
            Some(BlendCommands::Edit { hook_name }) => {
                edit_hook(hook_name)?;
            }
            Some(BlendCommands::List) => {
                list_hooks()?;
            }
            Some(BlendCommands::Run { hook_name, args }) => {
                run_hook(hook_name, args)?;
            }
            None => {
                if let Some(script_path) = &self.script_file {
                    add_hook(script_path, self.executable)?;
                } else {
                    return Err(anyhow!("Please provide a script file or use a subcommand"));
                }
            }
        }

        Ok(())
    }
}

fn get_hooks_dir() -> Result<PathBuf> {
    let home = home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    Ok(home.join(".pocket").join("hooks"))
}

fn get_hooks() -> Result<Vec<Hook>> {
    let hooks_dir = get_hooks_dir()?;
    if !hooks_dir.exists() {
        return Ok(Vec::new());
    }

    let mut hooks = Vec::new();
    for entry in fs::read_dir(hooks_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "sh") {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let executable = is_executable(&path)?;
            hooks.push(Hook { name, path, executable });
        }
    }

    Ok(hooks)
}

fn find_hook(hook_identifier: &str) -> Result<Hook> {
    let hooks = get_hooks()?;
    let search_name = if hook_identifier.starts_with('@') {
        &hook_identifier[1..]
    } else {
        hook_identifier
    };

    hooks
        .into_iter()
        .find(|hook| hook.name == search_name)
        .ok_or_else(|| anyhow!("Hook '{}' not found", search_name))
}

fn detect_shell_rc_file() -> Result<PathBuf> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let home = home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    
    if shell.contains("zsh") {
        Ok(home.join(".zshrc"))
    } else {
        // Default to bash if zsh is not detected
        Ok(home.join(".bashrc"))
    }
}

fn get_editor() -> Result<String> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string());
    
    Ok(editor)
}

fn is_executable(path: &Path) -> Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();
    Ok(permissions.mode() & 0o111 != 0)
}

fn add_hook(script_path: &str, executable: bool) -> Result<()> {
    let script_path = Path::new(script_path);
    if !script_path.exists() {
        return Err(anyhow!("Script file not found: {}", script_path.display()));
    }

    let hook_name = script_path
        .file_stem()
        .ok_or_else(|| anyhow!("Invalid script filename"))?
        .to_string_lossy();

    // Read the script content
    let mut script_content = String::new();
    File::open(script_path)?.read_to_string(&mut script_content)?;

    // Save the script to hooks directory
    let hooks_dir = get_hooks_dir()?;
    let hook_path = hooks_dir.join(format!("{}.sh", hook_name));
    
    fs::write(&hook_path, &script_content)
        .context(format!("Failed to write hook to {}", hook_path.display()))?;

    // Make executable if requested
    if executable {
        make_executable(&hook_path)?;
        create_hook_wrapper(&hook_name)?;
        println!("Successfully added executable hook '{}' from {}", hook_name, script_path.display());
        println!("You can run it with '@{}' or 'pocket blend run {}'", hook_name, hook_name);
    } else {
        // Add to shell rc file for sourcing
        add_hook_to_rc(&hook_name, &hook_path)?;
        println!("Successfully added hook '{}' from {}", hook_name, script_path.display());
        println!("Restart your shell or run 'source {}' to apply changes", detect_shell_rc_file()?.display());
    }

    Ok(())
}

fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o755); // rwxr-xr-x
    fs::set_permissions(path, perms)?;
    Ok(())
}

fn create_hook_wrapper(hook_name: &str) -> Result<()> {
    // Create a directory for executable wrappers
    let home = home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
    let bin_dir = home.join(".pocket").join("bin");
    fs::create_dir_all(&bin_dir)?;
    
    // Create the wrapper script
    let wrapper_path = bin_dir.join(format!("@{}", hook_name));
    let hooks_dir = get_hooks_dir()?;
    let hook_path = hooks_dir.join(format!("{}.sh", hook_name));
    
    let wrapper_content = format!(
        "#!/bin/bash\n\
         # Pocket CLI hook wrapper for {}\n\
         \"{0}\" \"$@\"\n",
        hook_path.display()
    );
    
    fs::write(&wrapper_path, wrapper_content)?;
    make_executable(&wrapper_path)?;
    
    // Add bin directory to PATH in shell config if not already there
    let rc_file = detect_shell_rc_file()?;
    let rc_content = fs::read_to_string(&rc_file)?;
    let path_line = format!("\n# Pocket CLI hook path\nexport PATH=\"$PATH:{}\"", bin_dir.display());
    
    if !rc_content.contains(&bin_dir.display().to_string()) {
        let mut file = OpenOptions::new().append(true).open(rc_file)?;
        file.write_all(path_line.as_bytes())?;
        println!("Added Pocket hook bin directory to your PATH");
    }
    
    Ok(())
}

fn add_hook_to_rc(hook_name: &str, hook_path: &PathBuf) -> Result<()> {
    let rc_file = detect_shell_rc_file()?;
    let hook_line = format!("\n# Pocket CLI hook: {}\nsource \"{}\"\n", hook_name, hook_path.display());
    
    // Check if the hook already exists in the rc file
    let rc_content = fs::read_to_string(&rc_file)?;
    if rc_content.contains(&format!("# Pocket CLI hook: {}", hook_name)) {
        println!("Hook '{}' already exists in {}", hook_name, rc_file.display());
        return Ok(());
    }
    
    // Append the hook to the rc file
    let mut file = OpenOptions::new().append(true).open(rc_file.clone())?;
    file.write_all(hook_line.as_bytes())?;
    
    println!("Added hook '{}' to {}", hook_name, rc_file.display());
    Ok(())
}

fn edit_hook(hook_identifier: &str) -> Result<()> {
    let hook = find_hook(hook_identifier)?;
    
    let editor = get_editor()?;
    println!("Opening {} with {}", hook.path.display(), editor);
    
    let status = Command::new(editor)
        .arg(&hook.path)
        .status()
        .context("Failed to launch editor")?;
    
    if status.success() {
        println!("Successfully edited hook '{}'", hook.name);
        Ok(())
    } else {
        Err(anyhow!("Editor exited with non-zero status"))
    }
}

fn run_hook(hook_identifier: &str, args: &[String]) -> Result<()> {
    let hook = find_hook(hook_identifier)?;
    
    if !hook.executable {
        return Err(anyhow!("Hook '{}' is not executable. Add with --executable flag to make it runnable.", hook.name));
    }
    
    println!("Running hook '{}'...", hook.name);
    
    let status = Command::new(&hook.path)
        .args(args)
        .status()
        .context(format!("Failed to execute hook '{}'", hook.name))?;
    
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("Hook '{}' exited with non-zero status: {}", hook.name, status))
    }
}

fn list_hooks() -> Result<()> {
    let hooks = get_hooks()?;
    
    if hooks.is_empty() {
        println!("No hooks installed");
        return Ok(());
    }
    
    println!("Installed hooks:");
    for hook in hooks {
        let hook_type = if hook.executable {
            "executable"
        } else {
            "shell extension"
        };
        println!("  @{} ({}) [{}]", hook.name, hook.path.display(), hook_type);
    }
    
    Ok(())
} 