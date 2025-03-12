//! Plugin architecture for Pocket CLI
//!
//! This module provides a plugin system for extending Pocket CLI functionality.
//! Plugins can add new commands, modify existing behavior, or provide additional features.

pub mod backup;

use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// Trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin
    fn name(&self) -> &str;
    
    /// Returns the version of the plugin
    fn version(&self) -> &str;
    
    /// Returns a description of the plugin
    fn description(&self) -> &str;
    
    /// Initializes the plugin with the given configuration
    fn initialize(&mut self, config: &PluginConfig) -> Result<()>;
    
    /// Executes a command provided by the plugin
    fn execute(&self, command: &str, args: &[String]) -> Result<()>;
    
    /// Returns a list of commands provided by the plugin
    fn commands(&self) -> Vec<PluginCommand>;
    
    /// Cleans up any resources used by the plugin
    fn cleanup(&mut self) -> Result<()>;
}

/// Configuration for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// The name of the plugin
    pub name: String,
    
    /// Whether the plugin is enabled
    pub enabled: bool,
    
    /// Additional configuration options for the plugin
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// A command provided by a plugin
#[derive(Debug, Clone)]
pub struct PluginCommand {
    /// The name of the command
    pub name: String,
    
    /// A description of the command
    pub description: String,
    
    /// The usage pattern for the command
    pub usage: String,
}

/// Manager for loading and running plugins
pub struct PluginManager {
    /// The loaded plugins
    plugins: Vec<Box<dyn Plugin>>,
    
    /// Configuration for each plugin
    configs: HashMap<String, PluginConfig>,
    
    /// The directory where plugins are stored
    plugin_dir: std::path::PathBuf,
}

impl PluginManager {
    /// Creates a new plugin manager with the given plugin directory
    pub fn new(plugin_dir: impl AsRef<Path>) -> Self {
        Self {
            plugins: Vec::new(),
            configs: HashMap::new(),
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Loads all plugins from the plugin directory
    pub fn load_plugins(&mut self) -> Result<()> {
        // Ensure the plugin directory exists
        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)
                .context("Failed to create plugin directory")?;
            return Ok(());
        }
        
        // Load plugin configurations
        self.load_configs()?;
        
        // Load dynamic libraries (this is a placeholder for actual dynamic loading)
        // In a real implementation, this would use libloading or similar to load plugins
        // For now, we'll just use built-in plugins
        
        // Register built-in plugins
        self.register_builtin_plugins()?;
        
        Ok(())
    }
    
    /// Loads plugin configurations from the plugin directory
    fn load_configs(&mut self) -> Result<()> {
        let config_path = self.plugin_dir.join("plugins.json");
        
        if !config_path.exists() {
            // Create a default configuration if none exists
            let default_configs: HashMap<String, PluginConfig> = HashMap::new();
            let json = serde_json::to_string_pretty(&default_configs)?;
            std::fs::write(&config_path, json)?;
            return Ok(());
        }
        
        // Read and parse the configuration file
        let json = std::fs::read_to_string(&config_path)?;
        self.configs = serde_json::from_str(&json)?;
        
        Ok(())
    }
    
    /// Saves plugin configurations to the plugin directory
    pub fn save_configs(&self) -> Result<()> {
        let config_path = self.plugin_dir.join("plugins.json");
        let json = serde_json::to_string_pretty(&self.configs)?;
        std::fs::write(&config_path, json)?;
        Ok(())
    }
    
    /// Registers built-in plugins
    fn register_builtin_plugins(&mut self) -> Result<()> {
        // Register the backup plugin
        use crate::plugins::backup::BackupPlugin;
        
        // Create the data directory path (parent of plugin_dir)
        let data_dir = self.plugin_dir.parent().unwrap_or(&self.plugin_dir).to_path_buf();
        
        // Register the backup plugin
        self.register_plugin(Box::new(BackupPlugin::new(data_dir)))?;
        
        Ok(())
    }
    
    /// Registers a plugin with the manager
    pub fn register_plugin(&mut self, mut plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        
        // Get or create a configuration for the plugin
        let config = self.configs.entry(name.clone()).or_insert_with(|| {
            PluginConfig {
                name: name.clone(),
                enabled: true,
                options: HashMap::new(),
            }
        });
        
        // Initialize the plugin with its configuration
        plugin.initialize(config)?;
        
        // Add the plugin to the list
        self.plugins.push(plugin);
        
        Ok(())
    }
    
    /// Returns a list of all loaded plugins
    pub fn list_plugins(&self) -> Vec<(&str, &str, bool)> {
        self.plugins.iter()
            .map(|p| {
                let name = p.name();
                let version = p.version();
                let enabled = self.configs.get(name)
                    .map(|c| c.enabled)
                    .unwrap_or(false);
                (name, version, enabled)
            })
            .collect()
    }
    
    /// Enables a plugin by name
    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = true;
            self.save_configs()?;
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }
    
    /// Disables a plugin by name
    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = false;
            self.save_configs()?;
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name)
        }
    }
    
    /// Executes a command provided by a plugin
    pub fn execute_command(&self, plugin_name: &str, command: &str, args: &[String]) -> Result<()> {
        // Find the plugin
        let plugin = self.plugins.iter()
            .find(|p| p.name() == plugin_name)
            .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", plugin_name))?;
        
        // Check if the plugin is enabled
        let enabled = self.configs.get(plugin_name)
            .map(|c| c.enabled)
            .unwrap_or(false);
        
        if !enabled {
            anyhow::bail!("Plugin '{}' is disabled", plugin_name);
        }
        
        // Execute the command
        plugin.execute(command, args)
    }
    
    /// Returns a list of all commands provided by all enabled plugins
    pub fn list_commands(&self) -> Vec<(String, Vec<PluginCommand>)> {
        self.plugins.iter()
            .filter(|p| {
                self.configs.get(p.name())
                    .map(|c| c.enabled)
                    .unwrap_or(false)
            })
            .map(|p| (p.name().to_string(), p.commands()))
            .collect()
    }
    
    /// Cleans up all plugins
    pub fn cleanup(&mut self) -> Result<()> {
        for plugin in &mut self.plugins {
            plugin.cleanup()?;
        }
        Ok(())
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        // Attempt to clean up plugins when the manager is dropped
        let _ = self.cleanup();
    }
} 