# Pocket Changelog

All notable changes to Pocket will be documented in this file using our letter-based versioning system.

## v-pocket-R1 (03102025 - Workflow Files)

### Added
- Support for `.pocket` workflow files in `~/.pocket/data/workflows/`
- Ability to run workflows by name without the `.pocket` extension
- Comments in workflow files for better documentation
- Automatic extraction of workflow descriptions from comments
- Directory listing of available workflows

### Changed
- Updated README with workflow file documentation
- Improved command parsing for workflow execution
- Enhanced error handling for workflow file operations

### Fixed
- Issue with workflow command parsing when using the save operation

## v-pocket-B2 (02152025 - Backpacks)

### Added
- Backpack feature for organizing related snippets
- Commands for creating and managing backpacks
- Ability to add snippets directly to backpacks
- Listing entries from specific backpacks

### Changed
- Storage structure to support backpack organization
- Command-line interface to include backpack options

## v-pocket-A1 (01052025 - Initial Release)

### Added
- Basic snippet storage and retrieval
- Search functionality with semantic matching
- Add, list, and remove commands
- Insert command for adding snippets to files
- Command-line interface with help documentation

### Notes
- First public release of Pocket
- Core functionality for managing code snippets

---

*Note: While this changelog uses our letter-based versioning system, the Cargo.toml file continues to use Semantic Versioning (SemVer) as required by the Rust ecosystem.*

## Version Naming Convention

Our versioning follows this format: `v-projectname-XN[-nc]`

Where:
- `v-projectname` identifies the project (pocket)
- `X` is a letter indicating stability:
  - `A` = Alpha (experimental, seeking feedback)
  - `B` = Beta (still buggy but usable for testing)
  - `C` = Candidate (almost ready for official release)
  - `R` = Release (stable and ready for production)
- `N` is a number indicating the iteration
- `-nc` (optional) indicates compatibility issues with previous versions

For example:
- `v-pocket-A1`: First alpha release
- `v-pocket-R2-nc`: Second stable release, not compatible with previous versions
- `v-pocket-R3-nc1<`: Third stable release, only compatible with version 1 and newer 