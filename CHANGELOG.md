# Pocket Changelog

All notable changes to Pocket will be documented in this file using our letter-based versioning system.

## v-pocket-R3B1-ncR2< (03252025 - Plugin System)

### Added
- Implemented plugin architecture for extending Pocket functionality
- Added backup plugin for creating and managing repository backups
- Added plugin commands to the CLI interface (list, enable, disable, execute)
- Support for plugin configuration management

### Changed
- Enhanced CLI interface to support plugin commands
- Improved error handling for plugin operations

### Notes
- Compatibility only extends to basic functions of Pocket and does not include any VCS support or Plugin support
- For VCS support, you will need a minimum of R3A1 (not recommended because it's alpha version) or R3A2 (also not recommended but better than A1)
- For plugin support, you will need R3B1 minimum and will be fully supported at a full R3 release

## v-pocket-R2A3-ncR3A1< (03212025 - Enhanced Version Control Features)

### Added
- Implemented proper graph command for visualizing timeline history
- Added ignore command for managing .pocketignore patterns
- Support for .pocketignore file to exclude files from version control

### Changed
- Refactored graph visualization to show actual repository timeline structure
- Enhanced error handling for graph and ignore commands

### Fixed
- Fixed issues with the pile command
- Improved timeline visualization in graph command

### Notes
- This version is only compatible with R3A1 and newer
- Enhances the VCS functionality with better visualization and ignore patterns

## v-pocket-R3A2-ncR3A1< (03202025 - Version Control System Improvements)

### Added
- Fixed pile command to save to the correct location
- Recursive directory support for pile command
- Enhanced help text for pile command
- Improved error handling for shove command

### Changed
- Updated pile command to handle directories recursively
- Fixed compatibility issues with previous version

### Notes
- This version is only compatible with R3A1 and newer
- Fixes critical issues with the pile and shove commands

## v-pocket-R3A1-nc (03152025 - Version Control System)

### Added
- Integrated custom Version Control System (VCS)
- Repository creation with `new-repo` command
- File staging with `pile` and `unpile` commands
- Commit functionality with `shove` command
- Branch management with `timeline` commands
- Repository status checking with `status` command
- History viewing with `log` command
- Merge functionality for timelines
- Remote repository management
- Improved help menu with categorized commands

### Changed
- Updated command-line interface to include VCS commands
- Enhanced help display to separate snippet management from version control
- Improved error handling for VCS operations

### Notes
- This is an alpha release of the VCS functionality
- Some VCS features may not work as expected
- The VCS implementation is still under active development

## v-pocket-R2B2-nc (03122025 - Bash Scripting Support)

### Added
- Automatic permission handling for script execution
- Support for executing non-executable scripts with automatic permission management
- Added `-f` flag to the execute command to specify script files
- Enhanced workflow support for script execution
- Python project setup workflow example
- Improved template insertion in workflows

### Changed
- Updated command parsing to better handle script arguments
- Enhanced error handling for script execution
- Improved workflow documentation with examples

### Fixed
- Fixed issues with script execution in workflows
- Improved handling of file paths in insert commands
- Fixed permission restoration after script execution

## v-pocket-R2B1 (03112025 - Enhanced Editor)

### Added
- Enhanced editor mode with syntax highlighting based on content type
- Automatic content type detection from file extensions and content patterns
- Language-specific templates for different content types
- Ability to edit existing entries with the new `edit` command
- Custom file extensions for better syntax highlighting in editors
- Smart selection of templates based on backpack names

### Changed
- Improved editor experience with better feedback to users
- More intelligent title extraction from content
- Detect and use user's preferred editor from system environment or configuration

### Fixed
- Fixed issues with temporary files not being properly cleaned up
- Improved error handling for editor interactions

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

## v-pocket-B2 (03092025 - Backpacks)

### Added
- Backpack feature for organizing related snippets
- Commands for creating and managing backpacks
- Ability to add snippets directly to backpacks
- Listing entries from specific backpacks

### Changed
- Storage structure to support backpack organization
- Command-line interface to include backpack options

## v-pocket-A1 (02122025 - Initial Release)

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
  - `