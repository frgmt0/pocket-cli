# <img src="assets/images/doggie.gif" width="40" alt="Pocket Dog"> Pocket

---

[![Hits-of-Code](https://hitsofcode.com/github/frgmt0/pocket-cli?branch=main)](https://hitsofcode.com/github/frgmt0/pocket-cli/view?branch=main)
[![Crates.io](https://img.shields.io/crates/v/pocket-cli)](https://crates.io/crates/pocket-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/frgmt0/pocket/pulls)

A CLI tool for saving, organizing, and retrieving code snippets with integrated version control. Because your brilliant solutions deserve better than being lost in a sea of Stack Overflow tabs.

## Overview

Pocket helps developers save and find code snippets, manage version control, and automate workflows. It's designed to reduce the time spent searching for code you've already written—because let's face it, you've solved this problem before, you just can't remember where you put the solution.

## Features

- **Snippet Management**: Save, search, and reuse code snippets that would otherwise be destined for digital oblivion
- **Semantic Search**: Find snippets using natural language queries, because remembering exact variable names from six months ago is unrealistic
- **Backpacks**: Organize snippets into collections, like folders but with a more adventurous name
- **Workflows**: Chain commands together for automation, because typing the same sequence repeatedly is soul-crushing
- **Cards**: Extend functionality with plugins, for when the base features aren't quite enough
- **Shell Integration**: 
  - Add shell extensions that load when your terminal starts, providing aliases and functions
  - Create executable hooks that can be run directly with the `@name` command syntax

## Quick Start

```bash
# Install (the beginning of a beautiful friendship)
cargo install pocket-cli

# Save a snippet (future you will thank present you)
pocket add file.js

# Find a snippet (with eerily human-like understanding)
pocket search "that thing with the loop"

# Use a snippet (the payoff moment)
pocket insert ID file.js

# Add a shell extension for aliases and functions
pocket blend my_aliases.sh

# Create an executable hook you can run with @name
pocket blend --executable my_script.sh
```

## Documentation

For more detailed information, check out the documentation, which we've actually taken the time to write properly:

- [Installation Guide](docs/installation.md) - Getting started without the headaches
- [Command Reference](docs/commands.md) - All the commands you'll forget and need to look up
- [Cards System](docs/cards.md) - Extending functionality without learning C++
- [Shell Hooks](docs/hooks.md) - Integrating Pocket with your shell environment

## Issues and Support

If you encounter any issues or have questions, please [create an issue](https://github.com/frgmt0/pocket/issues) on GitHub. Be sure to include details about your environment and the steps to reproduce the problem. The more specific you are, the faster we can help—vague bug reports are the digital equivalent of "it hurts when I do something."

## Join the discord
[Pocket CLI Discord Community](https://discord.gg/YDB5Kxf2xg)
  
## Version

Current version: `v-pocket-R1`

I use a versioning system with letter-based releases, because apparently semver was too mainstream. For more information, see my [versioning philosophy](https://blog.frgmt.xyz/03102025-tech).

![GitHub stars](https://img.shields.io/github/stars/frgmt0/pocket-cli?style=social)
![Crates.io Downloads](https://img.shields.io/crates/d/pocket-cli)
![GitHub issues](https://img.shields.io/github/issues/frgmt0/pocket-cli)

## License

MIT - Which means you can do pretty much whatever you want with this code, as long as you keep the copyright notice. Freedom comes with surprisingly little fine print sometimes.
