use crate::models::{Entry, Backpack, Config, ContentType, Workflow};
use anyhow::{Result, Context, anyhow};
use dirs::home_dir;
use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use chrono::Utc;

/// Storage manager for pocket data
#[derive(Clone)]
pub struct StorageManager {
    base_path: PathBuf,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new() -> Result<Self> {
        let base_path = Self::get_base_path()?;
        Ok(Self { base_path })
    }

    /// Get the base path for pocket data
    fn get_base_path() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let pocket_dir = home.join(".pocket");
        
        // Create directories if they don't exist
        create_dir_all(pocket_dir.join("data/entries"))?;
        create_dir_all(pocket_dir.join("data/backpacks"))?;
        create_dir_all(pocket_dir.join("data/workflows"))?;
        create_dir_all(pocket_dir.join("wallet"))?;
        
        Ok(pocket_dir)
    }

    /// Get the workflows directory
    pub fn _get_workflows_dir(&self) -> Result<PathBuf> {
        let dir = self.base_path.join("data/workflows");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Get the path for an entry's metadata
    fn get_entry_metadata_path(&self, id: &str, backpack: Option<&str>) -> PathBuf {
        match backpack {
            Some(name) => self.base_path.join(format!("data/backpacks/{}/entries/{}.json", name, id)),
            None => self.base_path.join(format!("data/entries/{}.json", id)),
        }
    }

    /// Get the path for an entry's content
    fn get_entry_content_path(&self, id: &str, backpack: Option<&str>) -> PathBuf {
        match backpack {
            Some(name) => self.base_path.join(format!("data/backpacks/{}/entries/{}.content", name, id)),
            None => self.base_path.join(format!("data/entries/{}.content", id)),
        }
    }

    /// Get the path for a backpack's metadata
    fn get_backpack_path(&self, name: &str) -> PathBuf {
        self.base_path.join(format!("data/backpacks/{}/manifest.json", name))
    }

    /// Get the config file path
    fn get_config_path(&self) -> PathBuf {
        self.base_path.join("config.toml")
    }

    /// Get the path to a workflow
    fn _get_workflow_path(&self, name: &str) -> PathBuf {
        self.base_path.join("data/workflows").join(format!("{}.json", name))
    }

    /// Save an entry to storage
    pub fn save_entry(&self, entry: &Entry, content: &str, backpack: Option<&str>) -> Result<()> {
        // Create backpack directory if needed
        if let Some(name) = backpack {
            create_dir_all(self.base_path.join(format!("data/backpacks/{}/entries", name)))?;
        }

        // Save metadata
        let metadata_path = self.get_entry_metadata_path(&entry.id, backpack);
        let metadata_json = serde_json::to_string_pretty(entry)?;
        fs::write(metadata_path, metadata_json)?;

        // Save content
        let content_path = self.get_entry_content_path(&entry.id, backpack);
        fs::write(content_path, content)?;

        Ok(())
    }

    /// Load an entry from storage
    pub fn load_entry(&self, id: &str, backpack: Option<&str>) -> Result<(Entry, String)> {
        // Load metadata
        let metadata_path = self.get_entry_metadata_path(id, backpack);
        let metadata_json = fs::read_to_string(&metadata_path)
            .with_context(|| format!("Failed to read entry metadata from {}", metadata_path.display()))?;
        let entry: Entry = serde_json::from_str(&metadata_json)
            .with_context(|| format!("Failed to parse entry metadata from {}", metadata_path.display()))?;

        // Load content
        let content_path = self.get_entry_content_path(id, backpack);
        let content = fs::read_to_string(&content_path)
            .with_context(|| format!("Failed to read entry content from {}", content_path.display()))?;

        Ok((entry, content))
    }

    /// Remove an entry from storage
    pub fn remove_entry(&self, id: &str, backpack: Option<&str>) -> Result<()> {
        // Remove metadata
        let metadata_path = self.get_entry_metadata_path(id, backpack);
        if metadata_path.exists() {
            fs::remove_file(&metadata_path)?;
        }

        // Remove content
        let content_path = self.get_entry_content_path(id, backpack);
        if content_path.exists() {
            fs::remove_file(&content_path)?;
        }

        Ok(())
    }

    /// List all entries in a backpack or the general pocket
    pub fn list_entries(&self, backpack: Option<&str>) -> Result<Vec<Entry>> {
        let entries_dir = match backpack {
            Some(name) => self.base_path.join(format!("data/backpacks/{}/entries", name)),
            None => self.base_path.join("data/entries"),
        };

        if !entries_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(entries_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only process JSON files (metadata)
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let metadata_json = fs::read_to_string(&path)?;
                let entry: Entry = serde_json::from_str(&metadata_json)?;
                entries.push(entry);
            }
        }

        // Sort by creation date (newest first)
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(entries)
    }

    /// Create a new backpack
    pub fn create_backpack(&self, backpack: &Backpack) -> Result<()> {
        // Create backpack directory
        let backpack_dir = self.base_path.join(format!("data/backpacks/{}", backpack.name));
        create_dir_all(backpack_dir.join("entries"))?;

        // Save backpack metadata
        let manifest_path = self.get_backpack_path(&backpack.name);
        let manifest_json = serde_json::to_string_pretty(backpack)?;
        fs::write(manifest_path, manifest_json)?;

        Ok(())
    }

    /// List all backpacks
    pub fn _list_backpacks(&self) -> Result<Vec<Backpack>> {
        let backpacks_dir = self.base_path.join("data/backpacks");
        let mut backpacks = Vec::new();
        
        for entry in fs::read_dir(&backpacks_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow!("Invalid backpack path"))?;
                
                // Each backpack is a subdirectory with entries
                let meta_path = path.join("manifest.json");
                if meta_path.exists() {
                    let meta_json = fs::read_to_string(&meta_path)?;
                    let backpack: Backpack = serde_json::from_str(&meta_json)?;
                    backpacks.push(backpack);
                } else {
                    backpacks.push(Backpack {
                        name: name.to_string(),
                        description: None,
                        created_at: Utc::now(),
                    });
                }
            }
        }
        
        Ok(backpacks)
    }

    /// Load the configuration
    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.get_config_path();
        
        if !config_path.exists() {
            // Create default config if it doesn't exist
            let config = Config::default();
            self.save_config(&config)?;
            return Ok(config);
        }

        let config_str = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        
        Ok(config)
    }

    /// Save the configuration
    pub fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.get_config_path();
        let config_str = toml::to_string_pretty(config)?;
        fs::write(config_path, config_str)?;
        
        Ok(())
    }

    /// Determine the content type from a file path
    pub fn _determine_content_type(path: &Path) -> ContentType {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                "rs" | "go" | "c" | "cpp" | "h" | "java" | "py" | "js" | "ts" => ContentType::Code,
                "md" | "txt" | "text" => ContentType::Text,
                "sh" | "bash" => ContentType::Script,
                _ => ContentType::Other(extension.to_string()),
            }
        } else {
            // If no extension, default to text
            ContentType::Text
        }
    }

    /// Save a workflow
    pub fn _save_workflow(&self, workflow: &Workflow) -> Result<()> {
        let workflow_path = self._get_workflow_path(&workflow.name);
        println!("Saving workflow to: {}", workflow_path.display());
        
        let workflow_json = serde_json::to_string_pretty(workflow)?;
        fs::write(workflow_path, workflow_json)?;
        
        Ok(())
    }
    
    /// Load a workflow
    pub fn _load_workflow(&self, name: &str) -> Result<Workflow> {
        let workflow_path = self._get_workflow_path(name);
        let workflow_json = fs::read_to_string(&workflow_path)
            .with_context(|| format!("Failed to read workflow '{}'", name))?;
        
        let workflow: Workflow = serde_json::from_str(&workflow_json)
            .with_context(|| format!("Failed to parse workflow '{}'", name))?;
        
        Ok(workflow)
    }
    
    /// Delete a workflow
    pub fn _delete_workflow(&self, name: &str) -> Result<()> {
        let workflow_path = self._get_workflow_path(name);
        if workflow_path.exists() {
            fs::remove_file(&workflow_path)?;
            Ok(())
        } else {
            Err(anyhow!("Workflow '{}' not found", name))
        }
    }
    
    /// List all workflows
    pub fn _list_workflows(&self) -> Result<Vec<Workflow>> {
        let dir = self.base_path.join("data/workflows");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut workflows = Vec::new();
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    match serde_json::from_str::<Workflow>(&content) {
                        Ok(workflow) => workflows.push(workflow),
                        Err(err) => log::warn!("Failed to parse workflow at {}: {}", path.display(), err),
                    }
                }
            }
        }
        
        // Sort by name
        workflows.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(workflows)
    }

    /// Search for entries by query string
    pub fn search_entries(&self, query: &str, backpack: Option<&str>, limit: usize) -> Result<Vec<(Entry, String)>> {
        let mut results = Vec::new();
        
        // Get entries to search
        let entries = self.list_entries(backpack)?;
        
        // Simple case-insensitive search
        let query_lower = query.to_lowercase();
        
        for entry in entries {
            // Load the content
            let content = match fs::read_to_string(self.get_entry_content_path(&entry.id, backpack)) {
                Ok(content) => content,
                Err(_) => continue, // Skip entries with missing content
            };
            
            // Check if query matches title or content
            if entry.title.to_lowercase().contains(&query_lower) || 
               content.to_lowercase().contains(&query_lower) {
                results.push((entry, content));
                
                // Check if we've reached the limit
                if results.len() >= limit {
                    break;
                }
            }
        }
        
        Ok(results)
    }
    
    /// Load the content of an entry
    pub fn _load_entry_content(&self, id: &str, backpack: Option<&str>) -> Result<String> {
        let content_path = self.get_entry_content_path(id, backpack);
        
        if !content_path.exists() {
            return Err(anyhow!("Content not found for entry '{}'", id));
        }
        
        let content = fs::read_to_string(&content_path)?;
        Ok(content)
    }
} 