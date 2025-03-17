use crate::errors::{PocketError, PocketResult, IntoPocketError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use once_cell::sync::OnceCell;
use log::{info, debug, error};

/// Global configuration instance
static CONFIG: OnceCell<Arc<Mutex<ConfigManager>>> = OnceCell::new();

/// Configuration for the Pocket CLI
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// Version of the configuration
    #[serde(default = "default_config_version")]
    pub version: String,
    
    /// Editor command to use (respects $EDITOR env var)
    #[serde(default)]
    pub editor: Option<String>,
    
    /// Default content type
    #[serde(default = "default_content_type")]
    pub default_content_type: String,
    
    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Path to the hooks directory
    #[serde(default)]
    pub hooks_dir: Option<PathBuf>,
    
    /// Path to the bin directory for executable hooks
    #[serde(default)]
    pub bin_dir: Option<PathBuf>,
    
    /// Maximum search results to display
    #[serde(default = "default_max_search_results")]
    pub max_search_results: usize,
    
    /// Search algorithm to use
    #[serde(default = "default_search_algorithm")]
    pub search_algorithm: String,
    
    /// Card configurations
    #[serde(default)]
    pub cards: HashMap<String, serde_json::Value>,
}

fn default_config_version() -> String {
    "1.0".to_string()
}

fn default_content_type() -> String {
    "Code".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_search_results() -> usize {
    10
}

fn default_search_algorithm() -> String {
    "fuzzy".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: default_config_version(),
            editor: None,
            default_content_type: default_content_type(),
            log_level: default_log_level(),
            hooks_dir: None,
            bin_dir: None,
            max_search_results: default_max_search_results(),
            search_algorithm: default_search_algorithm(),
            cards: HashMap::new(),
        }
    }
}

/// Manager for the Pocket CLI configuration
#[derive(Debug)]
pub struct ConfigManager {
    /// The configuration itself
    config: Config,
    
    /// Path to the configuration file
    config_path: PathBuf,
    
    /// Path to the data directory
    data_dir: PathBuf,
    
    /// Dirty flag to indicate if the config needs to be saved
    dirty: bool,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(data_dir: impl AsRef<Path>) -> PocketResult<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();
        let config_path = data_dir.join("config.toml");
        
        // Create a default config
        let config = if config_path.exists() {
            // Load the existing config
            let config_str = fs::read_to_string(&config_path)
                .config_err(&format!("Failed to read config file: {}", config_path.display()))?;
            
            match toml::from_str::<Config>(&config_str) {
                Ok(config) => {
                    debug!("Loaded config from {}", config_path.display());
                    config
                }
                Err(e) => {
                    error!("Failed to parse config file: {}", e);
                    error!("Using default config");
                    Config::default()
                }
            }
        } else {
            info!("No config file found, creating default config");
            Config::default()
        };
        
        let config_exists = config_path.exists();
        
        let manager = Self {
            config,
            config_path,
            data_dir,
            dirty: false,
        };
        
        // Save the default config if it doesn't exist
        if !config_exists {
            manager.save()?;
        }
        
        Ok(manager)
    }
    
    /// Get the configuration
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }
    
    /// Update the configuration
    pub fn update_config(&mut self, config: Config) -> PocketResult<()> {
        self.config = config;
        self.dirty = true;
        Ok(())
    }
    
    /// Get a value from the card configuration
    pub fn get_card_config(&self, card_name: &str) -> Option<serde_json::Value> {
        self.config.cards.get(card_name).cloned()
    }
    
    /// Set a value in the card configuration
    pub fn set_card_config(&mut self, card_name: &str, config: serde_json::Value) -> PocketResult<()> {
        self.config.cards.insert(card_name.to_string(), config);
        self.dirty = true;
        Ok(())
    }
    
    /// Save the configuration to disk
    pub fn save(&self) -> PocketResult<()> {
        let config_str = toml::to_string_pretty(&self.config)
            .config_err("Failed to serialize config")?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .config_err(&format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        // Write the config file
        fs::write(&self.config_path, config_str)
            .config_err(&format!("Failed to write config to {}", self.config_path.display()))?;
        
        debug!("Saved config to {}", self.config_path.display());
        Ok(())
    }
    
    /// Get the data directory
    pub fn get_data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }
    
    /// Get the hooks directory
    pub fn get_hooks_dir(&self) -> PathBuf {
        match &self.config.hooks_dir {
            Some(dir) => dir.clone(),
            None => self.data_dir.join("hooks"),
        }
    }
    
    /// Get the bin directory
    pub fn get_bin_dir(&self) -> PathBuf {
        match &self.config.bin_dir {
            Some(dir) => dir.clone(),
            None => self.data_dir.join("bin"),
        }
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        if self.dirty {
            match self.save() {
                Ok(_) => {}
                Err(e) => error!("Failed to save config on drop: {}", e),
            }
        }
    }
}

/// Initialize the global configuration
pub fn init(data_dir: impl AsRef<Path>) -> PocketResult<()> {
    let config_manager = ConfigManager::new(data_dir)?;
    let _ = CONFIG.set(Arc::new(Mutex::new(config_manager)));
    Ok(())
}

/// Get the global configuration manager
pub fn get() -> PocketResult<Arc<Mutex<ConfigManager>>> {
    match CONFIG.get() {
        Some(config) => Ok(config.clone()),
        None => Err(PocketError::Config("Configuration not initialized".to_string())),
    }
}

/// Get a copy of the current configuration
pub fn get_config() -> PocketResult<Config> {
    let config = get()?;
    let config_guard = config.lock()
        .map_err(|_| PocketError::Config("Failed to lock config".to_string()))?;
    Ok(config_guard.get_config())
}

/// Save the current configuration
pub fn save() -> PocketResult<()> {
    let config = get()?;
    let config_guard = config.lock()
        .map_err(|_| PocketError::Config("Failed to lock config".to_string()))?;
    config_guard.save()
} 