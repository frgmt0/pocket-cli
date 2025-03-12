# Version Control System

Pocket includes a built-in version control system that works similarly to Git but with a simpler interface and terminology.

## Key Concepts

- **Repository**: A collection of files and their history
- **Pile**: The staging area (similar to Git's index)
- **Shove**: A commit or snapshot of your files
- **Timeline**: A branch or line of development
- **Remote**: A connection to another repository

## Basic Workflow

### Creating a Repository

```bash
pocket new-repo [path]
```

This initializes a new repository in the specified path (or current directory if not specified).

### Checking Status

```bash
pocket status
```

Shows the current state of your repository, including modified files and the current timeline.

### Adding Files to the Pile

```bash
# Add specific files
pocket pile file1.txt file2.js

# Add all files
pocket pile --all

# Add files matching a pattern
pocket pile --pattern "*.js"
```

### Creating a Shove (Commit)

```bash
# With a message
pocket shove -m "Your commit message"

# Open editor for message
pocket shove -e
```

### Viewing History

```bash
# Basic log
pocket log

# Detailed log
pocket log --verbose
```

## Working with Timelines (Branches)

### Creating a Timeline

```bash
pocket timeline new timeline_name
```

### Switching Timelines

```bash
pocket timeline switch timeline_name
```

### Listing Timelines

```bash
pocket timeline list
```

### Merging Timelines

```bash
pocket merge timeline_name
```

## Working with Remotes

### Adding a Remote

```bash
pocket remote add remote_name url
```

### Listing Remotes

```bash
pocket remote list
```

### Fetching from a Remote

```bash
pocket fish [remote_name]
```

### Pushing to a Remote

```bash
pocket push [remote_name] [timeline_name]
```

## Ignoring Files

```bash
# Add a pattern to ignore
pocket ignore --pattern "*.log"

# Remove a pattern
pocket ignore --remove "*.log"

# List ignore patterns
pocket ignore --list
``` 