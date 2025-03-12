//! Card architecture for Pocket CLI
//!
//! This module provides a card system for extending Pocket CLI functionality.
//! Cards can add new commands, modify existing behavior, or provide additional features.

pub mod backup;

use std::collections::HashMap;
use std::path::Path;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

/// Trait that all cards must implement
pub trait Card: Send + Sync {
    /// Returns the name of the card
    fn name(&self) -> &str;
    
    /// Returns the version of the card
    fn version(&self) -> &str;
    
    /// Returns a description of the card
    fn description(&self) -> &str;
    
    /// Initializes the card with the given configuration
    fn initialize(&mut self, config: &CardConfig) -> Result<()>;
    
    /// Executes a command provided by the card
    fn execute(&self, command: &str, args: &[String]) -> Result<()>;
    
    /// Returns a list of commands provided by the card
    fn commands(&self) -> Vec<CardCommand>;
    
    /// Cleans up any resources used by the card
    fn cleanup(&mut self) -> Result<()>;
}

/// Configuration for a card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardConfig {
    /// The name of the card
    pub name: String,
    
    /// Whether the card is enabled
    pub enabled: bool,
    
    /// Additional configuration options for the card
    #[serde(default)]
    pub options: HashMap<String, serde_json::Value>,
}

/// A command provided by a card
#[derive(Debug, Clone)]
pub struct CardCommand {
    /// The name of the command
    pub name: String,
    
    /// A description of the command
    pub description: String,
    
    /// The usage pattern for the command
    pub usage: String,
}

/// Manager for loading and running cards
pub struct CardManager {
    /// The loaded cards
    cards: Vec<Box<dyn Card>>,
    
    /// Configuration for each card
    configs: HashMap<String, CardConfig>,
    
    /// The directory where cards are stored
    card_dir: std::path::PathBuf,
}

impl CardManager {
    /// Creates a new card manager with the given card directory
    pub fn new(card_dir: impl AsRef<Path>) -> Self {
        Self {
            cards: Vec::new(),
            configs: HashMap::new(),
            card_dir: card_dir.as_ref().to_path_buf(),
        }
    }
    
    /// Loads all cards from the card directory
    pub fn load_cards(&mut self) -> Result<()> {
        // Ensure the card directory exists
        if !self.card_dir.exists() {
            std::fs::create_dir_all(&self.card_dir)
                .context("Failed to create card directory")?;
            return Ok(());
        }
        
        // Load card configurations
        self.load_configs()?;
        
        // Register built-in cards
        self.register_builtin_cards()?;
        
        // Load dynamic cards from wallet directory
        let wallet_dir = self.card_dir.parent().unwrap_or(&self.card_dir).join("wallet");
        if wallet_dir.exists() {
            for entry in std::fs::read_dir(wallet_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    let card_name = path.file_name()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| anyhow::anyhow!("Invalid card directory name"))?;
                    
                    // Check if this card is already registered in the configuration
                    if !self.configs.contains_key(card_name) {
                        // Register the card configuration
                        self.register_card_config(card_name, "local")?;
                    }
                    
                    // Check if the card has been built
                    let target_dir = path.join("target").join("release");
                    let lib_name = format!("libpocket_card_{}.dylib", card_name.replace("-", "_"));
                    let lib_path = target_dir.join(lib_name);
                    
                    if lib_path.exists() {
                        // TODO: In a real implementation, we would load the dynamic library here
                        // For now, just print a message
                        println!("Found built card at {}", lib_path.display());
                        
                        // Check if the card is already loaded
                        if !self.cards.iter().any(|p| p.name() == card_name) {
                            // For now, we'll just add a placeholder card
                            // In a real implementation, we would load the dynamic library
                            // and create a proper card instance
                            
                            // For testing purposes, let's create a placeholder card
                            let card = Box::new(PlaceholderCard::new(card_name.to_string()));
                            self.register_card(card)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Loads card configurations from the card directory
    fn load_configs(&mut self) -> Result<()> {
        let config_path = self.card_dir.join("cards.json");
        
        if !config_path.exists() {
            // Create a default configuration if none exists
            let default_configs: HashMap<String, CardConfig> = HashMap::new();
            let json = serde_json::to_string_pretty(&default_configs)?;
            std::fs::write(&config_path, json)?;
            return Ok(());
        }
        
        // Read and parse the configuration file
        let json = std::fs::read_to_string(&config_path)?;
        self.configs = serde_json::from_str(&json)?;
        
        Ok(())
    }
    
    /// Saves card configurations to the card directory
    pub fn save_configs(&self) -> Result<()> {
        let config_path = self.card_dir.join("cards.json");
        let json = serde_json::to_string_pretty(&self.configs)?;
        std::fs::write(&config_path, json)?;
        Ok(())
    }
    
    /// Registers built-in cards
    fn register_builtin_cards(&mut self) -> Result<()> {
        // Register the backup card
        use crate::cards::backup::BackupCard;
        
        // Create the data directory path (parent of card_dir)
        let data_dir = self.card_dir.parent().unwrap_or(&self.card_dir).to_path_buf();
        
        // Register the backup card
        self.register_card(Box::new(BackupCard::new(data_dir)))?;
        
        Ok(())
    }
    
    /// Registers a card with the manager
    pub fn register_card(&mut self, mut card: Box<dyn Card>) -> Result<()> {
        let name = card.name().to_string();
        
        // Get or create a configuration for the card
        let config = self.configs.entry(name.clone()).or_insert_with(|| {
            CardConfig {
                name: name.clone(),
                enabled: true,
                options: HashMap::new(),
            }
        });
        
        // Initialize the card with its configuration
        card.initialize(config)?;
        
        // Add the card to the list
        self.cards.push(card);
        
        Ok(())
    }
    
    /// Returns a list of all loaded cards
    pub fn list_cards(&self) -> Vec<(&str, &str, bool)> {
        self.cards.iter()
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
    
    /// Enables a card by name
    pub fn enable_card(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = true;
            self.save_configs()?;
            Ok(())
        } else {
            anyhow::bail!("Card '{}' not found", name)
        }
    }
    
    /// Disables a card by name
    pub fn disable_card(&mut self, name: &str) -> Result<()> {
        if let Some(config) = self.configs.get_mut(name) {
            config.enabled = false;
            self.save_configs()?;
            Ok(())
        } else {
            anyhow::bail!("Card '{}' not found", name)
        }
    }
    
    /// Executes a command on a card
    pub fn execute_command(&self, card_name: &str, command: &str, args: &[String]) -> Result<()> {
        // Find the card
        let card = self.cards.iter().find(|p| p.name() == card_name);
        
        if let Some(card) = card {
            // Check if the card is enabled
            let enabled = self.configs.get(card_name)
                .map(|c| c.enabled)
                .unwrap_or(false);
            
            if !enabled {
                return Err(anyhow::anyhow!("Card '{}' is disabled", card_name));
            }
            
            // Execute the command
            card.execute(command, args)
        } else {
            // Check if the card exists in the configuration but is not loaded
            if self.configs.contains_key(card_name) {
                // For now, just return an error indicating the card is not loaded
                // In a real implementation, we would attempt to load the card dynamically
                return Err(anyhow::anyhow!("Card '{}' is registered but not loaded. Try rebuilding the card with: pocket cards build {}", card_name, card_name));
            }
            
            Err(anyhow::anyhow!("Card '{}' not found", card_name))
        }
    }
    
    /// Returns a list of all commands provided by all enabled cards
    pub fn list_commands(&self) -> Vec<(String, Vec<CardCommand>)> {
        self.cards.iter()
            .filter(|p| {
                self.configs.get(p.name())
                    .map(|c| c.enabled)
                    .unwrap_or(false)
            })
            .map(|p| (p.name().to_string(), p.commands()))
            .collect()
    }
    
    /// Cleans up all cards
    pub fn cleanup(&mut self) -> Result<()> {
        for card in &mut self.cards {
            card.cleanup()?;
        }
        Ok(())
    }
    
    /// Checks if a card with the given name exists
    pub fn card_exists(&self, name: &str) -> bool {
        // Check if the card is in the loaded cards
        if self.cards.iter().any(|p| p.name() == name) {
            return true;
        }
        
        // Check if the card is in the configuration
        self.configs.contains_key(name)
    }
    
    /// Registers a card configuration without loading the card
    pub fn register_card_config(&mut self, name: &str, url: &str) -> Result<()> {
        // Create a new configuration for the card
        let config = CardConfig {
            name: name.to_string(),
            enabled: true,
            options: {
                let mut options = HashMap::new();
                options.insert("url".to_string(), serde_json::Value::String(url.to_string()));
                options
            },
        };
        
        // Add the configuration
        self.configs.insert(name.to_string(), config);
        
        // Save the configurations
        self.save_configs()?;
        
        Ok(())
    }
    
    /// Removes a card configuration
    pub fn remove_card_config(&mut self, name: &str) -> Result<()> {
        // Remove the card from the configuration
        self.configs.remove(name);
        
        // Save the configurations
        self.save_configs()?;
        
        Ok(())
    }
}

impl Drop for CardManager {
    fn drop(&mut self) {
        // Attempt to clean up cards when the manager is dropped
        let _ = self.cleanup();
    }
}

// A placeholder card for testing
struct PlaceholderCard {
    name: String,
    version: String,
    description: String,
}

impl PlaceholderCard {
    fn new(name: String) -> Self {
        Self {
            name,
            version: "0.1.0".to_string(),
            description: "A placeholder card".to_string(),
        }
    }
}

impl Card for PlaceholderCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn initialize(&mut self, _config: &CardConfig) -> Result<()> {
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        println!("Executing command {} with args {:?} on placeholder card {}", command, args, self.name);
        Ok(())
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "hello".to_string(),
                description: "A simple hello command".to_string(),
                usage: format!("pocket cards execute {} hello [args...]", self.name),
            },
        ]
    }
    
    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
} 