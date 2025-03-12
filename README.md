# 🦘 Pocket

A CLI tool for saving, organizing, and retrieving code snippets with integrated version control.

## Overview

Pocket helps developers save and find code snippets, manage version control, and automate workflows. It's designed to reduce the time spent searching for code you've already written.

## Features

- **Snippet Management**: Save, search, and reuse code snippets
- **Semantic Search**: Find snippets using natural language queries
- **Backpacks**: Organize snippets into collections
- **Version Control**: Built-in Git-like version control system
- **Workflows**: Chain commands together for automation
- **Cards**: Extend functionality with plugins

## Quick Start

```bash
# Install
cargo install pocket-cli

# Save a snippet
pocket add file.js

# Find a snippet
pocket search "that thing with the loop"

# Use a snippet
pocket insert ID file.js
```

## Documentation

For more detailed information, check out the documentation:

- [Installation Guide](docs/installation.md)
- [Command Reference](docs/commands.md)
- [Version Control System](docs/version-control.md)
- [Cards System](docs/cards.md)

## Issues and Support

If you encounter any issues or have questions, please [create an issue](https://github.com/frgmt0/pocket/issues) on GitHub. Be sure to include details about your environment and the steps to reproduce the problem.

## Version

Current version: `0.6.2`

We use a versioning system that combines semantic versioning with letter-based releases. For more information, see our [versioning guide](https://blog.frgmt.xyz/03102025-tech).

## License

MIT