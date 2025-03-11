# Pocket Version Control System (VCS) Specification

## Overview

This document specifies the integration of a custom version control system into the Pocket CLI tool. This is not a Git wrapper but a standalone, purpose-built VCS designed to complement Pocket's existing functionality while introducing novel approaches to version control.

The Pocket VCS aims to provide a simpler, more intuitive version control experience with modern features that address common pain points in existing systems. While maintaining conceptual familiarity for users of Git, it introduces innovations that improve workflow and reduce cognitive load.

## Core Concepts

### Laundry Rooms ("Rooms")

A Pocket rooms is a directory containing files under version control. Unlike Git, Pocket rooms are designed to be more self-contained and user-friendly:

- `.pocket/` directory contains all version control data (similar to `.git/`)
- Repository metadata is stored in a structured, human-readable format
- No detached HEAD states or complex concepts that confuse beginners

### Piles

A "pile" is the staging area concept (similar to Git's index/staging area). Files are "piled up" before being committed:

- Each pile is a collection of changes that will form a commit
- Piles can be modified, split, combined, or abandoned before committing
- Piles have a more visual representation to users than Git's staging area

### Shoves

A "shove" is a commit - a snapshot of changes that have been "piled up" and then permanently stored in the repository:

- Each shove has a unique ID, timestamp, author, and message
- Shoves form a directed acyclic graph (DAG) like Git commits
- Shoves can be tagged, annotated, and organized into collections

### Timelines

Instead of branches, Pocket uses "timelines" - parallel development paths that can diverge and merge:

- Timelines are more visually intuitive than branches
- Default timeline is "main" (rather than "master")
- Timelines are designed to be created and merged with less friction

### Snapshots

A snapshot is a complete picture of the repository at a specific point in time (a shove). Users can "back" to any snapshot to restore the repository to that state.

### UX

pocket's VCS will still be within the terminal, but it would be nice to have pretty icons especially when things are branching out, or viewing timelines, or diffs. it would make the UX more appealing

## Command Specification

### Repository Management

#### `pocket new repo`

Create a new Pocket repository in the current directory.

**Usage:**
```bash
pocket new repo [--template <template-name>]
```

**Options:**
- `--template <template-name>`: Initialize with a predefined template (no defined templates yet)
- `--no-default`: Don't create default files like README.md or .pocketignore (usually we want the default files to show up but it depends)

**Behavior:**
- Creates a `.pocket/` directory with necessary subdirectories
- Initializes an empty repository with no shoves
- Creates a default "main" timeline
- Sets up user configuration based on global settings

**Structure Created:**
```
.pocket/
  ├── config.toml       # Repository configuration
  ├── timelines/        # Timeline definitions
  │   └── main.toml     # Default timeline
  ├── objects/          # Stored file objects (content-addressable)
  ├── shoves/           # Commit history
  ├── piles/            # Staging area state
  └── snapshots/        # Named snapshots for easy referencing
```

#### `pocket status`

Display the current state of the repository.

**Usage:**
```bash
pocket status [--verbose]
```

**Output:**
- Current timeline
- Files in the current pile (staged changes)
- Unstaged changes
- Recent shoves
- Conflicts (if any)

### Change Management

#### `pocket pile`

Add files to the current pile (staging area).

**Usage:**
```bash
pocket pile [file1] [file2] ... # Specific files
pocket pile --all               # All changes
pocket pile --pattern "*.rs"    # Files matching pattern
```

**Options:**
- `--all`: Add all changes (similar to `git add .`)
- `--interactive`: Interactive mode to select chunks
- `--pattern <pattern>`: Add files matching the glob pattern

**Behavior:**
- Tracks both new files and changes to existing files
- Creates file snapshots in the `.pocket/piles/` directory
- Updates the pile index
- Shows a summary of what was added to the pile

#### `pocket unpile`

Remove files from the current pile (unstage changes).

**Usage:**
```bash
pocket unpile [file1] [file2] ... # Specific files
pocket unpile --all               # All staged changes
```

**Options:**
- `--all`: Remove all files from the pile

#### `pocket pile-diff`

Show the difference between what's in the pile and the last shove.

**Usage:**
```bash
pocket pile-diff [file]
```

#### `pocket shove`

Create a permanent snapshot (commit) from the current pile.

**Usage:**
```bash
pocket shove -m "Commit message"  # With inline message
pocket shove -e                   # Open editor for message
```

**Options:**
- `-m, --message <msg>`: Specify commit message inline
- `-e, --editor`: Open the default editor for the message
- `--tag <tag-name>`: Immediately tag this shove
- `--no-verify`: Skip any pre-shove hooks

**Behavior:**
- Creates a new shove object in `.pocket/shoves/`
- Generates a unique ID for the shove
- Updates the current timeline to point to the new shove
- Clears the pile after successful shove
- Records author, timestamp, and message

#### `pocket back`

Return the repository to a previous state (snapshot/shove).

**Usage:**
```bash
pocket back <shove-id>            # Back to specific shove
pocket back --shoves 3            # Back 3 shoves
pocket back --tag <tag-name>      # Back to a tagged shove
```

**Options:**
- `--soft`: Keep changes in the working directory
- `--hard`: Discard all changes in the working directory
- `--pile`: Keep changes but add them to the pile

**Behavior:**
- Updates working directory to reflect the state at the specified shove
- Moves the timeline pointer to the specified shove
- Can create a new timeline automatically if backing to a non-head shove

### Timeline Management

#### `pocket timeline`

List, create, or switch timelines (branches).

**Usage:**
```bash
pocket timeline                   # List all timelines
pocket timeline new <name>        # Create new timeline
pocket timeline switch <name>     # Switch to timeline
```

**Options:**
- `--based-on <shove-id>`: Create timeline from specific shove
- `--track`: Set up tracking relationship for remote timelines

#### `pocket merge`

Merge another timeline into the current one.

**Usage:**
```bash
pocket merge <timeline>
```

**Options:**
- `--strategy <strategy>`: Specify merge strategy
- `--no-shove`: Don't automatically create a merge shove

### History and Inspection

#### `pocket log`

Show the shove history.

**Usage:**
```bash
pocket log [--graph] [--limit <n>]
```

**Options:**
- `--graph`: Show ASCII graph visualization
- `--limit <n>`: Limit to n entries
- `--timeline <name>`: Show history for specific timeline
- `--format <format>`: Custom format for output

#### `pocket show`

Show details about a specific shove.

**Usage:**
```bash
pocket show <shove-id>
```

**Output:**
- Shove ID
- Author and timestamp
- Full message
- List of changed files
- Diff of changes

### Collaboration

#### `pocket remote`

Manage remote repositories.

**Usage:**
```bash
pocket remote add <name> <url>    # Add remote
pocket remote remove <name>       # Remove remote
pocket remote list                # List remotes
```

#### `pocket fish`

Get updates from a remote repository.

**Usage:**
```bash
pocket fish [remote]
```

#### `pocket yank`

Fetch and merge changes from a remote timeline.

**Usage:**
```bash
pocket yank [remote] [timeline]
```

#### `pocket push`

Send local timelines to a remote repository.

**Usage:**
```bash
pocket push [remote] [timeline]
```

## Innovative Features

### 1. Smart Conflict Resolution

Unlike Git's conflict markers that modify files directly, Pocket uses a smarter approach:

- **Conflict Files**: Conflicts are stored separately in `.pocket/conflicts/` without modifying working files
- **Visual Conflict Resolution**: Built-in tools to visualize and resolve conflicts
- **Resolution Suggestions**: AI-assisted conflict resolution that suggests the most likely correct merge

```bash
pocket resolve [file]  # Interactive conflict resolution
```

### 2. Undo Anything

Pocket maintains a comprehensive history of all operations, making it possible to undo virtually any action:

```bash
pocket undo                # Undo last operation
pocket redo                # Redo previously undone operation
pocket undo --list         # List undo history
pocket undo --to <id>      # Undo multiple operations
```

This works for all operations including:
- Shoves
- Merges
- Timeline switches
- Pile operations

### 3. Time Machine Mode

A more intuitive way to navigate repository history:

```bash
pocket timemachine
```

This launches an interactive mode where users can:
- See a visualization of the repository over time
- Scroll through different shoves to see the state at any point
- Extract files or content from any historical state without switching the whole repo
- Compare any two points in time visually

### 4. Smart Ignores

Instead of requiring a `.pocketignore` file, Pocket has context-aware ignore patterns (although users can include a `.pocketignore` if they'd like):

- Automatically detects project type and suggests appropriate ignores
- Learns from user behavior which files are frequently ignored
- Separates temporary ignores from permanent ones

```bash
pocket ignore add "*.log"          # Add a permanent ignore pattern
pocket ignore temp "node_modules/" # Ignore temporarily
pocket ignore suggest              # Get suggestions based on project
```

### 5. Shove Squashing and Reordering

Unlike Git's complex rebase operation, Pocket offers intuitive history manipulation:

```bash
pocket combine <from-id> <to-id>    # Combine range of shoves
pocket reorder                      # Interactive timeline reordering
```

The UI makes it clear what is happening, with visual representations of the before and after states.

### 6. Timeline Management

Improved workflow for parallel development:

- **Timeline Groups**: Organize related timelines together
- **Timeline Stacks**: Push/pop timeline states like a stack
- **Smart Merges**: Detect when a timeline can be fast-forwarded automatically

```bash
pocket timeline group create "feature-x"
pocket timeline stack save "quick-fix"
pocket timeline stack pop
```

### 7. Partial Shoves (pretty big)

Commit only parts of files without complex patch selection:

```bash
pocket pile --lines file.rs:10-25  # Add only lines 10-25
pocket pile --function file.rs:myFunction # Add only a specific function
```

Built-in understanding of programming language syntax allows for function/class level tracking.

## Technical Implementation

### Storage Format

Unlike Git's loose objects and packfiles, Pocket uses a more structured approach:

1. **Shove Storage**:
   - Shoves are stored as structured JSON/TOML files
   - File content is stored using content-addressable storage
   - Metadata is stored separately from content for better performance

2. **Delta Compression**:
   - Changes between versions are stored as deltas
   - Periodic full snapshots prevent delta chains from becoming too long
   - Binary files use specialized delta algorithms

3. **Indexing**:
   - B-tree indexes for fast lookup of files, shoves, and objects
   - Full-text search capability across shove messages and file contents
   - Timeline-specific indexes for improved performance

### Core Components (Rust Implementation)

#### Repository Module
```rust
pub struct Repository {
    path: PathBuf,
    config: Config,
    current_timeline: Timeline,
    pile: Pile,
}

impl Repository {
    pub fn new(path: &Path) -> Result<Self>;
    pub fn open(path: &Path) -> Result<Self>;
    pub fn status(&self) -> RepoStatus;
    // Additional methods...
}
```

#### Pile Module
```rust
pub struct Pile {
    base_shove: Option<ShoveId>,
    entries: HashMap<PathBuf, PileEntry>,
}

pub struct PileEntry {
    status: PileStatus,
    object_id: ObjectId,
    original_path: PathBuf,
}

impl Pile {
    pub fn add_path(&mut self, path: &Path) -> Result<()>;
    pub fn remove_path(&mut self, path: &Path) -> Result<()>;
    pub fn clear(&mut self) -> Result<()>;
    // Additional methods...
}
```

#### Shove Module
```rust
pub struct Shove {
    id: ShoveId,
    parent_ids: Vec<ShoveId>,
    author: Author,
    timestamp: DateTime<Utc>,
    message: String,
    root_tree_id: ObjectId,
}

impl Shove {
    pub fn new(
        pile: &Pile,
        parent_ids: Vec<ShoveId>,
        author: Author,
        message: &str,
    ) -> Result<Self>;
    
    pub fn get_changes(&self) -> Result<Vec<FileChange>>;
    // Additional methods...
}
```

#### Timeline Module
```rust
pub struct Timeline {
    name: String,
    head: ShoveId,
    remote: Option<RemoteTracking>,
}

impl Timeline {
    pub fn new(name: &str, head: ShoveId) -> Self;
    pub fn switch_to(&mut self, repo: &Repository) -> Result<()>;
    pub fn merge(&mut self, other: &Timeline) -> Result<MergeResult>;
    // Additional methods...
}
```

#### Object Storage Module
```rust
pub struct ObjectStore {
    base_path: PathBuf,
}

impl ObjectStore {
    pub fn store_file(&self, path: &Path) -> Result<ObjectId>;
    pub fn get_file(&self, id: &ObjectId) -> Result<Vec<u8>>;
    pub fn store_tree(&self, tree: &Tree) -> Result<ObjectId>;
    // Additional methods...
}
```

### Database Schema

Pocket uses a combination of file-system storage and embedded database:

1. **Config Database**:
   - Repository settings
   - User preferences
   - Ignore patterns

2. **Object Database**:
   - Content-addressable storage for file contents
   - Tree structures representing directories
   - Delta-compressed storage for efficient space usage

3. **Timeline Database**:
   - Timeline definitions and metadata
   - Relationships between timelines
   - Remote tracking information

4. **Shove Database**:
   - Shove metadata (author, timestamp, message)
   - Parent-child relationships
   - Tags and annotations

## Security Considerations

### Cryptographic Verification

- All shoves can be cryptographically signed
- Content integrity is verified using secure hashes
- Support for verifying external contributions

### Access Control

- Fine-grained permissions for who can modify timelines
- Protected timelines that require approval for changes
- Audit logging of all operations

### Secure Collaboration

- Built-in support for SSH and HTTPS protocols
- Credential management without storing plaintext passwords
- Integration with external authentication systems

## Migration from Git (mildly important since we have no way to host the repo other than on github or similar)

To ease adoption, Pocket provides Git migration tools:

```bash
pocket import-git [path]  # Import a Git repository
pocket export-git [path]  # Export to Git format
```

The importer preserves:
- Full commit history
- Branches and tags
- Author information
- Signed commits

## Future Extensions

### Distributed Workflow Enhancement

- Improved handling of simultaneous changes
- Better conflict prediction and prevention
- Offline-first approach with robust synchronization

### Large File Handling

- Transparent handling of large binary files
- Chunking and deduplicated storage
- Lazy loading of large repositories

### Integration with Pocket's Core Features

- Version control for snippets in pocket storage
- Applying versioned snippets to files
- Timeline-aware snippet search

## Conclusion

The Pocket VCS provides a fresh approach to version control that maintains familiarity for Git users while introducing innovations that make version control more intuitive and less error-prone. By focusing on real-world developer workflows and pain points, Pocket VCS aims to be a significant improvement over existing solutions while maintaining the power and flexibility expected from a modern version control system.