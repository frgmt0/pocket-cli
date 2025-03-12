# Add from a file
`pocket add path/to/brilliant/solution.js`

# Add directly from your thoughts
`pocket add -m "SELECT * FROM users WHERE sanity > 0;"`

# Open your editor when inspiration strikes
`pocket add -e`

# Open your editor with syntax highlighting specific to a backpack
`pocket add -e -b javascript`

Later, when you need to find that solution, you don't need to remember exactly what you called it or where you put it. You can search with natural language, describing the problem it solves rather than its exact syntax:

```bash
# Search with natural language
pocket search "that pagination thing with the cursor"

# Edit an existing entry
pocket edit c5358b8f
```

## Command Reference

Here are the main commands you'll use with Pocket:

```bash
# Adding content
pocket add [FILE]                    # Add from a file
pocket add -m "content"              # Add directly
pocket add -e                        # Open in your editor with smart templates
pocket add -e -b backpack-name       # Open editor with language-specific highlighting

# Finding content
pocket search "query"                # Search your snippets
pocket list                          # List all entries
pocket list --include-backpacks      # List entries in all backpacks

# Using content
pocket insert [ID] [FILE]            # Insert a snippet into a file
pocket remove [ID]                   # Remove a snippet
pocket edit [ID]                     # Edit a snippet in your editor

# Organization
pocket create backpack NAME          # Create a new backpack
pocket list --backpack NAME          # List entries in a backpack

# Workflows
pocket lint "command > command"      # Run a command chain
pocket lint workflow-name            # Run a .pocket workflow file
pocket lint                          # List available workflows

# Other
pocket version                       # Show version information
```

## Using Plugins

Pocket supports a plugin system that extends its functionality. Plugins add new commands and features to help you manage your snippets more effectively.

### Managing Plugins

```bash
# List all available plugins
pocket plugins list

# Enable a plugin
pocket plugins enable <plugin-name>

# Disable a plugin
pocket plugins disable <plugin-name>
```

### Backup Plugin

The backup plugin helps you create, manage, and restore backups of your snippets and repositories.

```bash
# Create a backup with an optional description
pocket backup [description]

# List all available backups
pocket backup list

# Restore a backup
pocket backup restore <backup-id>

# Delete a backup
pocket backup delete <backup-id>
```

#### Example Backup Workflow

1. Create a regular backup before making significant changes:
   ```bash
   pocket backup "Before reorganizing my JavaScript snippets"
   ```

2. View your available backups:
   ```bash
   pocket backup list
   ```
   This will show a list of backups with their IDs, descriptions, creation dates, and content statistics.

3. If something goes wrong, restore from a backup:
   ```bash
   pocket backup restore backup_20230615_123045
   ```
   This will restore your snippets and repositories to the state they were in when the backup was created.

4. Clean up old backups you no longer need:
   ```bash
   pocket backup delete backup_20230101_090000
   ```

By default, the backup plugin automatically maintains a limited number of backups, keeping only the most recent ones to save disk space.

For example:
- `v-pocket-A1`: First alpha release
- `v-pocket-R2-nc`: Second stable release, not compatible with previous versions
- `v-pocket-R3-nc1<`: Third stable release, only compatible with version 1 and newer

Current: `v-pocket-R3A2-ncR3A1<`