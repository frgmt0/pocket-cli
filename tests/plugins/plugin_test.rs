//! Tests for the plugin system
//!
//! These tests verify that the plugin system works correctly.

use std::path::Path;
use assert_fs::prelude::*;
use predicates::prelude::*;

use pocket_cli::plugins::{Plugin, PluginConfig, PluginCommand, PluginManager};

// Import common test utilities
use crate::common;

// A simple test plugin for testing the plugin system
struct TestPlugin {
    name: String,
    version: String,
    description: String,
    initialized: bool,
    executed_commands: Vec<(String, Vec<String>)>,
}

impl TestPlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            initialized: false,
            executed_commands: Vec::new(),
        }
    }
}

impl Plugin for TestPlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn initialize(&mut self, _config: &PluginConfig) -> anyhow::Result<()> {
        self.initialized = true;
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> anyhow::Result<()> {
        // Clone the command and args for testing
        let mut plugin = self.clone();
        plugin.executed_commands.push((command.to_string(), args.to_vec()));
        Ok(())
    }
    
    fn commands(&self) -> Vec<PluginCommand> {
        vec![
            PluginCommand {
                name: "test".to_string(),
                description: "A test command".to_string(),
                usage: "pocket test-plugin test".to_string(),
            },
            PluginCommand {
                name: "echo".to_string(),
                description: "Echoes the arguments".to_string(),
                usage: "pocket test-plugin echo [args...]".to_string(),
            },
        ]
    }
    
    fn cleanup(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// Clone implementation for TestPlugin
impl Clone for TestPlugin {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            initialized: self.initialized,
            executed_commands: self.executed_commands.clone(),
        }
    }
}

#[test]
fn test_plugin_manager_creation() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let plugin_dir = temp.path();
    
    // Create a plugin manager
    let manager = PluginManager::new(plugin_dir);
    
    // Verify that the plugin directory was created
    assert!(plugin_dir.exists(), "Plugin directory should exist");
}

#[test]
fn test_plugin_registration() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let plugin_dir = temp.path();
    
    // Create a plugin manager
    let mut manager = PluginManager::new(plugin_dir);
    
    // Create a test plugin
    let plugin = TestPlugin::new("test-plugin");
    
    // Register the plugin
    manager.register_plugin(Box::new(plugin)).expect("Failed to register plugin");
    
    // List plugins
    let plugins = manager.list_plugins();
    
    // Verify that the plugin was registered
    assert_eq!(plugins.len(), 1, "Should have exactly one plugin");
    assert_eq!(plugins[0].0, "test-plugin", "Plugin name should be 'test-plugin'");
    assert_eq!(plugins[0].1, "1.0.0", "Plugin version should be '1.0.0'");
    assert_eq!(plugins[0].2, true, "Plugin should be enabled");
}

#[test]
fn test_plugin_enable_disable() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let plugin_dir = temp.path();
    
    // Create a plugin manager
    let mut manager = PluginManager::new(plugin_dir);
    
    // Create and register test plugins
    let plugin1 = TestPlugin::new("plugin1");
    let plugin2 = TestPlugin::new("plugin2");
    
    manager.register_plugin(Box::new(plugin1)).expect("Failed to register plugin1");
    manager.register_plugin(Box::new(plugin2)).expect("Failed to register plugin2");
    
    // Disable plugin1
    manager.disable_plugin("plugin1").expect("Failed to disable plugin1");
    
    // List plugins
    let plugins = manager.list_plugins();
    
    // Verify that plugin1 is disabled and plugin2 is enabled
    let plugin1_info = plugins.iter().find(|p| p.0 == "plugin1").expect("Plugin1 not found");
    let plugin2_info = plugins.iter().find(|p| p.0 == "plugin2").expect("Plugin2 not found");
    
    assert_eq!(plugin1_info.2, false, "Plugin1 should be disabled");
    assert_eq!(plugin2_info.2, true, "Plugin2 should be enabled");
    
    // Enable plugin1
    manager.enable_plugin("plugin1").expect("Failed to enable plugin1");
    
    // List plugins again
    let plugins = manager.list_plugins();
    
    // Verify that both plugins are enabled
    let plugin1_info = plugins.iter().find(|p| p.0 == "plugin1").expect("Plugin1 not found");
    let plugin2_info = plugins.iter().find(|p| p.0 == "plugin2").expect("Plugin2 not found");
    
    assert_eq!(plugin1_info.2, true, "Plugin1 should be enabled");
    assert_eq!(plugin2_info.2, true, "Plugin2 should be enabled");
}

#[test]
fn test_plugin_command_execution() {
    // Create a temporary directory for the test
    let temp = common::create_temp_dir();
    let plugin_dir = temp.path();
    
    // Create a plugin manager
    let mut manager = PluginManager::new(plugin_dir);
    
    // Create and register a test plugin
    let plugin = TestPlugin::new("test-plugin");
    
    manager.register_plugin(Box::new(plugin)).expect("Failed to register plugin");
    
    // Execute a command
    let args = vec!["arg1".to_string(), "arg2".to_string()];
    manager.execute_command("test-plugin", "test", &args).expect("Failed to execute command");
    
    // List commands
    let commands = manager.list_commands();
    
    // Verify that the plugin provides the expected commands
    assert_eq!(commands.len(), 1, "Should have commands from exactly one plugin");
    assert_eq!(commands[0].0, "test-plugin", "Commands should be from 'test-plugin'");
    assert_eq!(commands[0].1.len(), 2, "Plugin should provide exactly two commands");
    assert_eq!(commands[0].1[0].name, "test", "First command should be 'test'");
    assert_eq!(commands[0].1[1].name, "echo", "Second command should be 'echo'");
} 