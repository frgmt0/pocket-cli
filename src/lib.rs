//! Pocket CLI library
//! 
//! This file exports all the modules needed by the binary and tests.

// Re-export modules
pub mod cards;
pub mod cli;
pub mod config;
pub mod errors;
pub mod logging;
pub mod models;
pub mod search;
pub mod storage;
pub mod utils;
pub mod version;

// Re-export frequently used items
pub use errors::{PocketError, PocketResult};
pub use config::Config;

// Add any other modules that need to be accessible to tests 