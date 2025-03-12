# Installation Guide

## Prerequisites

- Rust and Cargo (1.70.0 or newer)
- Git (for version control features)

## Installing from Cargo

The easiest way to install Pocket is through Cargo:

```bash
cargo install pocket-cli
```

This will download and compile the latest stable version of Pocket.

## Building from Source

If you want to build from source:

```bash
# Clone the repository
git clone https://github.com/frgmt0/pocket.git
cd pocket

# Build the release version
cargo build --release

# Optional: Add to your PATH
cp target/release/pocket /usr/local/bin/  # Linux/macOS
# or
copy target\release\pocket.exe %USERPROFILE%\bin\  # Windows
```

## Verifying Installation

To verify that Pocket is installed correctly:

```bash
pocket version
```

This should display the current version of Pocket.

## Directory Structure

Pocket stores all data in `~/.pocket/` with the following structure:

- `data/entries/` - General snippets
- `data/backpacks/` - Organized collections of snippets
- `data/workflows/` - Saved command chains
- `cards/` - Configuration for cards (plugins)
- `wallet/` - Installed cards (plugins) 