# Pocket CLI Installation Guide
*Getting started without the headaches*

This guide walks you through installing Pocket CLI on your system with minimal fuss. Let's get you up and running quickly.

## Prerequisites

Before diving in, make sure you have:

- **Rust and Cargo** (version 1.70.0 or newer) - The foundation of our Rust-based tool
- **Git** - Required for version control features and, well, modern developer existence

If you're missing either of these, now's the time to install them. We'll wait.

## Installation Options

### The Standard Approach: Cargo

The simplest way to install Pocket is through Cargo, Rust's package manager:

```bash
cargo install pocket-cli
```

This command downloads, compiles, and installs the latest stable release of Pocket CLI. Cargo handles all the dependencies and puts the binary in the right place.

### The DIY Approach: Building from Source

For those who prefer to see how the sausage is made (or want the very latest features):

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

Building from source gives you the latest features and fixes, but might occasionally include experimental functionality.

## Verification

To confirm that Pocket has been properly installed:

```bash
pocket version
```

You should see the current version number displayed. If not, check that the installation directory is in your PATH.

## Directory Structure

Pocket keeps its data in `~/.pocket/` with the following structure:

- `data/entries/` - Your general snippets and code fragments
- `data/backpacks/` - Organized collections for different projects
- `data/workflows/` - Saved command sequences for automation
- `cards/` - Configuration files for your plugins
- `wallet/` - The actual plugin code lives here

Understanding this structure will help if you ever need to manually manage your Pocket data or troubleshoot issues.

## Next Steps

Now that you have Pocket CLI installed, check out the Command Reference guide to start organizing your code snippets or the Cards System guide to extend functionality with plugins.

Congratulations - you're ready to start bringing order to your code snippets and development workflows.