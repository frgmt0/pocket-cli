//! Main test file for Pocket CLI
//!
//! This file imports and runs all the tests for the project.

// Import common test utilities
pub mod common;

// Import VCS tests
mod vcs {
    pub mod repository_test;
    pub mod remote_test;
}

// Import storage tests
mod storage {
    pub mod snippet_test;
}

// Import plugin tests
mod plugins {
    pub mod plugin_test;
}

// Import integration tests
mod integration {
    pub mod vcs_snippet_integration_test;
} 