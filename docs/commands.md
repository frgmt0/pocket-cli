# Command Reference

## Snippet Management

### Adding Content

```bash
# Add from a file
pocket add file.js

# Add with inline text
pocket add -m "code goes here"

# Open editor to add content
pocket add -e

# Add to a specific backpack
pocket add -b backpack_name file.js
```

### Finding Content

```bash
# List all entries
pocket list

# List entries in a specific backpack
pocket list --backpack backpack_name

# Search for entries
pocket search "query"

# Search with exact matching
pocket search "query" --exact

# Search in a specific backpack
pocket search "query" --backpack backpack_name
```

### Using Content

```bash
# Insert a snippet into a file
pocket insert ID file.js

# Remove a snippet
pocket remove ID

# Edit a snippet
pocket edit ID
```

## Organization

### Backpacks

```bash
# Create a new backpack
pocket create backpack name

# List entries in a backpack
pocket list --backpack name
```

### Workflows

```bash
# Execute a workflow
pocket lint "search 'query' > insert file.js"

# Save a workflow
pocket lint --save workflow_name "search 'query' > insert file.js"

# Execute a saved workflow
pocket lint workflow_name

# Delete a workflow
pocket delete-workflow workflow_name
```

## Version Control

```bash
# Initialize a repository
pocket new-repo

# Show repository status
pocket status

# Add files to staging area
pocket pile file.js
pocket pile --all

# Remove files from staging area
pocket unpile file.js
pocket unpile --all

# Create a commit
pocket shove -m "Commit message"
pocket shove -e  # Open editor for message

# Show commit history
pocket log
pocket log --verbose

# Create a new timeline (branch)
pocket timeline new name

# Switch to a timeline
pocket timeline switch name

# List timelines
pocket timeline list

# Merge a timeline
pocket merge timeline_name

# Add a remote repository
pocket remote add name url

# Push to a remote
pocket push remote_name
```

## Cards (Plugins)

```bash
# List available cards
pocket cards list

# Add a card from GitHub
pocket cards add name url

# Create a new local card
pocket cards create name "description"

# Build a card
pocket cards build name

# Run a card command
pocket cards run card_name command [args...]

# Enable/disable a card
pocket cards enable name
pocket cards disable name

# Remove a card
pocket cards remove name
```

## General

```bash
# Display help
pocket help
pocket help command

# Display version information
pocket version
``` 