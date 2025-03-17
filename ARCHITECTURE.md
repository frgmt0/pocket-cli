# Pocket CLI Architecture

This document outlines the architecture of the Pocket CLI, a tool for saving, organizing, and retrieving code snippets with integrated version control and shell integration capabilities.

## Overview

The Pocket CLI is designed with a modular, extensible architecture that allows for easy addition of new features through cards (plugins). The codebase is organized into a set of core modules that provide the foundational functionality, with additional features implemented as cards.

## Core Modules

### `main.rs`
The entry point for the application. It handles parsing command-line arguments and delegates to the appropriate card for handling the command.

### `cli/`
Contains the command-line interface definition and command handler:
- `mod.rs`: Defines the CLI structure using Clap
- `handler.rs`: Handles routing of commands to the appropriate card

### `errors.rs`
Defines the error types and utilities for error handling throughout the application.

### `logging.rs`
Provides a standardized logging and console output system.

### `config.rs`
Manages application configuration, with support for global and card-specific settings.

### `models/`
Contains the data models used throughout the application:
- `Entry`: Represents a saved snippet or code
- `Backpack`: Represents a collection of entries
- `ContentType`: Enum for different types of content

### `storage/`
Handles persisting and retrieving data:
- `StorageManager`: Central manager for all storage operations

### `search/`
Provides search functionality across entries:
- `SearchEngine`: Implements search algorithms (fuzzy, semantic)

### `utils/`
Contains utility functions used throughout the application.

### `cards/`
The card system, which is the extensibility mechanism:
- `mod.rs`: Core card system implementation
- Individual card implementations (e.g., `core.rs`, `blend.rs`, etc.)

## Card System

The card system is the primary extensibility mechanism in Pocket CLI. Each card is a module that provides a set of related functionalities:

### Card Trait
All cards implement the `Card` trait, which defines:
- Basic metadata (name, version, description)
- Initialization and cleanup methods
- Command execution handling
- Command listing

### Built-in Cards

#### Core Card (`core.rs`)
Handles core functionality:
- Searching for entries
- Listing entries
- Creating backpacks
- Removing entries
- Inserting entries into files

#### Blend Card (`blend.rs`)
Handles shell integration:
- Adding shell extensions (sourced at shell startup)
- Creating executable hooks (run with @name)
- Managing hooks (edit, list, run)

#### Snippet Card (`snippet.rs`)
Handles snippet management:
- Adding snippets from files, clipboard, or editor
- Summarizing content

#### Backup Card (`backup.rs`)
Handles backup and versioning:
- Creating backups
- Restoring from backups
- Managing versions

### Card Registration and Discovery
- Cards are registered in `cards/mod.rs`
- The `CardManager` class handles card loading, initialization, and command routing

## Data Flow

1. User enters a command (`pocket ...`)
2. `main.rs` parses the command using Clap
3. `cli/handler.rs` routes the command to the appropriate card
4. The card executes the command and returns a result
5. `main.rs` handles the result (displays output, errors, etc.)

## Configuration

The configuration system is based on TOML files:
- Global configuration in `~/.pocket/config.toml`
- Card-specific configuration in the same file, under card-specific keys

The `ConfigManager` provides a centralized way to access and modify configuration.

## Error Handling

Errors are handled through the `PocketError` enum, which defines different error types:
- Storage errors
- Entry errors
- CLI errors
- Card errors
- etc.

The `PocketResult` type alias is used throughout the codebase for consistent error handling.

## Logging

The logging system is based on the standard `log` crate, with custom formatting:
- `logging::init()` initializes the logger
- Log levels are configurable through the `--verbose` flag or configuration
- User-friendly output functions are provided in the `logging` module

## Future Extensions

The architecture is designed to be extended with new cards. To add a new card:
1. Create a new file in `src/cards/` that implements the `Card` trait
2. Register the card in `src/cards/mod.rs`
3. Add any necessary configuration options in `config.rs`
4. Add commands to the CLI in `cli/mod.rs`

## Development Practices

- Use the error handling system consistently
- Add proper logging at appropriate levels
- Update configuration when adding new features
- Write tests for all new functionality
- Document changes in CHANGELOG.md
- Follow Rust best practices (formatting, linting, etc.) 