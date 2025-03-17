use thiserror::Error;
use std::path::PathBuf;

/// Primary error type for the Pocket CLI application
#[derive(Error, Debug)]
pub enum PocketError {
    /// Error related to storage operations
    #[error("Storage error: {0}")]
    Storage(String),

    /// Error related to specific entry operations
    #[error("Entry error: {0}")]
    Entry(String),

    /// Error related to CLI operations
    #[error("CLI error: {0}")]
    Cli(String),

    /// Error related to card operations
    #[error("Card error: {0}")]
    Card(String),

    /// Error related to hook/blend operations
    #[error("Hook error: {0}")]
    Hook(String),

    /// Error related to file operations
    #[error("File error: {source}")]
    File {
        #[source]
        source: std::io::Error,
        path: PathBuf,
    },

    /// Error related to configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error related to search operations
    #[error("Search error: {0}")]
    Search(String),

    /// User canceled an operation
    #[error("Operation canceled by user")]
    Canceled,

    /// Missing permission
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Other unexpected errors
    #[error("Unexpected error: {0}")]
    Other(String),
}

/// Result type alias for Pocket CLI
pub type PocketResult<T> = std::result::Result<T, PocketError>;

/// Helper functions for converting errors
pub trait IntoAnyhow<T> {
    fn into_anyhow(self) -> anyhow::Result<T>;
}

impl<T> IntoAnyhow<T> for PocketResult<T> {
    fn into_anyhow(self) -> anyhow::Result<T> {
        self.map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}

/// Helper trait for converting errors to PocketError
pub trait IntoPocketError<T> {
    fn storage_err(self, msg: &str) -> PocketResult<T>;
    fn entry_err(self, msg: &str) -> PocketResult<T>;
    fn card_err(self, msg: &str) -> PocketResult<T>;
    fn hook_err(self, msg: &str) -> PocketResult<T>;
    fn config_err(self, msg: &str) -> PocketResult<T>;
    fn search_err(self, msg: &str) -> PocketResult<T>;
    fn other_err(self, msg: &str) -> PocketResult<T>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> IntoPocketError<T> for Result<T, E> {
    fn storage_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Storage(format!("{}: {}", msg, e)))
    }

    fn entry_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Entry(format!("{}: {}", msg, e)))
    }

    fn card_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Card(format!("{}: {}", msg, e)))
    }

    fn hook_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Hook(format!("{}: {}", msg, e)))
    }

    fn config_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Config(format!("{}: {}", msg, e)))
    }

    fn search_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Search(format!("{}: {}", msg, e)))
    }

    fn other_err(self, msg: &str) -> PocketResult<T> {
        self.map_err(|e| PocketError::Other(format!("{}: {}", msg, e)))
    }
} 