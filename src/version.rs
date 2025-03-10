/// Version information for Pocket
/// 
/// This module provides version information in both SemVer (for Cargo)
/// and our letter-based versioning system that prioritizes communication

/// The current version in letter-based format
pub const VERSION_LETTER: &str = "v-pocket-R1";

/// The current version as a date string (MMDDYYYY) - for internal tracking
pub const VERSION_DATE: &str = "03102025";

/// The current version as a human-readable string
pub const VERSION_STRING: &str = "Pocket v-pocket-R1 (03102025 - Workflow Files)";

/// Compatibility information
pub const COMPATIBILITY: Option<&str> = None; // None means fully compatible

/// Get the current version as a letter-based string
pub fn get_version_letter() -> &'static str {
    VERSION_LETTER
}

/// Get the current version as a date string (for internal tracking)
pub fn get_version_date() -> &'static str {
    VERSION_DATE
}

/// Get the current version as a human-readable string
pub fn get_version_string() -> &'static str {
    VERSION_STRING
}

/// Get the current version as a structured object
pub fn get_version() -> Version {
    Version {
        letter: VERSION_LETTER,
        date: VERSION_DATE,
        semver: env!("CARGO_PKG_VERSION"),
        name: "Workflow Files",
        compatibility: COMPATIBILITY,
        stability: Stability::Release,
    }
}

/// Version stability levels
pub enum Stability {
    /// Alpha: Experimental and seeking feedback
    Alpha,
    
    /// Beta: Still buggy but not completely unusable
    Beta,
    
    /// Candidate: Almost ready for official release
    Candidate,
    
    /// Release: Stable and ready for production use
    Release,
}

impl std::fmt::Display for Stability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stability::Alpha => write!(f, "Alpha"),
            Stability::Beta => write!(f, "Beta"),
            Stability::Candidate => write!(f, "Candidate"),
            Stability::Release => write!(f, "Release"),
        }
    }
}

/// Version information structure
pub struct Version {
    /// Version in letter-based format (e.g., v-pocket-R1)
    pub letter: &'static str,
    
    /// Version as a date string (MMDDYYYY) - for internal tracking
    pub date: &'static str,
    
    /// SemVer version from Cargo.toml (required for Rust ecosystem)
    pub semver: &'static str,
    
    /// Name of this version/release
    pub name: &'static str,
    
    /// Compatibility information (None means fully compatible)
    pub compatibility: Option<&'static str>,
    
    /// Stability level
    pub stability: Stability,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.letter)?;
        
        if let Some(compat) = self.compatibility {
            write!(f, "-{}", compat)?;
        }
        
        write!(f, " ({})", self.name)
    }
} 