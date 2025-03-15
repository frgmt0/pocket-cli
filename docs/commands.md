# Pocket CLI Command Reference

## Snippet Management
*Because copy-paste is so 2005*

### Adding Content
*Getting your code bits into the system*

```bash
# Add from a file (the normal way)
pocket add file.js

# Add with inline text (for the impatient)
pocket add -m "console.log('why am I hard-coding this?');"

# Open editor to add content (for the thoughtful types)
pocket add -e

# Add to a specific backpack (organization level: advanced)
pocket add -b backpack_name file.js
```

### Finding Content
*Because what's the point of saving if you can't find it later?*

```bash
# List ALL THE THINGS
pocket list

# List entries in a specific backpack
pocket list --backpack backpack_name

# Search for entries (basic mode)
pocket search "query"

# Search with exact matching (perfectionist mode)
pocket search "query" --exact

# Search in a specific backpack (when you've actually organized things)
pocket search "query" --backpack backpack_name
```

### Using Content
*The payoff for all that meticulous hoarding*

```bash
# Insert a snippet into a file (the magic moment)
pocket insert ID file.js

# Remove a snippet (spring cleaning)
pocket remove ID

# Edit a snippet (because perfection is a process)
pocket edit ID
```

## Organization
*Tools for people who have their life together, or aspire to*

### Backpacks
*Like folders, but more adventurous*

```bash
# Create a new backpack (for when one messy pile isn't enough)
pocket create backpack name

# List entries in a backpack (see what's inside)
pocket list --backpack name
```

### Workflows
*For when you're tired of typing the same commands repeatedly*

```bash
# Execute a workflow (one-liner automation)
pocket lint "search 'query' > insert file.js"

# Save a workflow (for future laziness)
pocket lint --save workflow_name "search 'query' > insert file.js"

# Execute a saved workflow (maximum efficiency)
pocket lint workflow_name

# Delete a workflow (spring cleaning, automation edition)
pocket delete-workflow workflow_name
```

## Version Control
*Git, but with more... something? i'm not sure*
NOTE: STILL IN ALPHA/BETA!!!!! Use at your own risk!

```bash
# Initialize a repository (the beginning of your journey)
pocket new-repo

# Show repository status (what mess have I created?)
pocket status

# Add files to staging area (single file mode)
pocket pile file.js
# Add files to staging area (sweep everything under the rug mode)
pocket pile --all

# Remove files from staging area (single file regret)
pocket unpile file.js
# Remove files from staging area (total regret)
pocket unpile --all

# Create a commit (freeze your work in digital amber)
pocket shove -m "Fixed the thing that wasn't working"
pocket shove -e  # For when your commit deserves an essay

# Show commit history (trip down memory lane)
pocket log
pocket log --verbose  # For when you want ALL the details

# Create a new timeline (branch) (parallel universe mode)
pocket timeline new name

# Switch to a timeline (universe hopping)
pocket timeline switch name

# List timelines (see all your parallel universes)
pocket timeline list

# FUTURE Merge a timeline (universe collision)
pocket merge timeline_name

# FUTURE Add a remote repository (make friends with other computers)
pocket remote add name url

# FUTURE Push to a remote (share your genius)
pocket push remote_name
```

## Cards (Plugins)
*Extend functionality without learning C++*

```bash
# List available cards (see what toys you have)
pocket cards list

# Add a card from GitHub (trust someone else's code)
pocket cards add name url

# Create a new local card (DIY mode)
pocket cards create name "description"

# Build a card (make your code actually work)
pocket cards build name

# Run a card command (the moment of truth)
pocket cards run card_name command [args...]

# Enable/disable a card (power management)
pocket cards enable name
pocket cards disable name

# Remove a card (breakup time)
pocket cards remove name
```

## General
*The meta stuff*

```bash
# Display help (admit defeat)
pocket help
pocket help command

# Display version information (for bragging rights)
pocket version
```

## Utility Commands
*Making your CLI life easier*

### Shell Integration with Blend
*Mixing your shell scripts into your environment*

```bash
# Add a shell script as a shell extension (sourced at shell startup)
pocket blend my_aliases.sh

# Add a shell script as an executable hook (can be run with @name)
pocket blend --executable my_script.sh

# List all installed hooks
pocket blend list

# Edit an existing hook
pocket blend edit hook_name

# Run an executable hook
pocket blend run hook_name [arguments]
# Or, after shell restart:
@hook_name [arguments]
```

The `blend` command provides two ways to integrate scripts with your shell:

1. **Shell Extensions**: Scripts added without the `--executable` flag are sourced when your shell starts, making aliases and functions available in your terminal.

2. **Executable Hooks**: Scripts added with the `--executable` flag can be directly executed using the `@name` prefix or via `pocket blend run`.

For more details, check out the [Shell Hooks](hooks.md) documentation.

## The Fine Print

Remember, with great Pocket power comes great responsibility. These commands can make your coding life dramatically better—or at least more organized, which is basically the same thing. If something breaks, well... you've got version control, right?

Happy pocketing!