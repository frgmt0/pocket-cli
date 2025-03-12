# Pocket Tests

This directory contains tests for the Pocket application, including both unit tests and integration tests.

## Directory Structure

- `unit/`: Contains unit tests for individual components
  - `vcs/`: Tests for version control system components
  - `snippets/`: Tests for snippet management components
- `integration/`: Contains integration tests that test multiple components together
- `common/`: Contains shared utilities for tests

## Running Tests

To run all tests:

```bash
cargo test
```

To run only unit tests:

```bash
cargo test --test unit
```

To run only integration tests:

```bash
cargo test --test integration
```

To run a specific test:

```bash
cargo test test_name
```

## Writing Tests

When writing new tests:

1. Place unit tests in the appropriate subdirectory under `unit/`
2. Place integration tests in the `integration/` directory
3. Use the utilities in `common/mod.rs` for common operations
4. Follow the existing test patterns for consistency

## Test Coverage

The tests in this directory aim to cover:

1. Core VCS functionality (repository, commits, timelines, etc.)
2. Snippet management (storage, retrieval, search)
3. Integration between VCS and snippet management
4. Workflow functionality
5. Ignore patterns and file exclusion

## Adding New Tests

When adding new features to Pocket, please also add corresponding tests to ensure the feature works correctly and continues to work in the future. 