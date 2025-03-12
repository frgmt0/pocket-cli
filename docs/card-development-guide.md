# Card Development Guide

This guide provides detailed instructions for developing custom Cards (plugins) for Pocket CLI.

## Overview

Cards are Pocket's plugin system that allows you to extend the functionality of the CLI with custom commands and features. Cards are implemented as Rust libraries that implement the `Card` trait.

## Prerequisites

- Rust and Cargo (1.70.0 or newer)
- Basic understanding of Rust programming
- Pocket CLI installed

## Creating a New Card

### Using the Built-in Command

The easiest way to create a new card is using the built-in command:

```bash
pocket cards create my-card "Description of my card"
```

This will create a new card in your wallet directory (`~/.pocket/wallet/my-card`) with the following structure:

```
my-card/
├── Cargo.toml
├── card.toml
├── README.md
└── src/
    └── lib.rs
```

### Manual Creation

If you prefer to create the card manually, you can create the directory structure yourself:

1. Create a directory in `~/.pocket/wallet/` with your card name
2. Create the necessary files as described below

## Card Structure

### Cargo.toml

The `Cargo.toml` file should include:

```toml
[package]
name = "pocket-card-my-card"
version = "0.1.0"
edition = "2021"
description = "Description of my card"
authors = ["Your Name <your.email@example.com>"]
license = "MIT"

[lib]
name = "pocket_card_my_card"
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Note the `crate-type = ["cdylib"]` which is required for dynamic loading.

### card.toml

The `card.toml` file contains metadata about your card:

```toml
[card]
name = "my-card"
version = "0.1.0"
description = "Description of my card"
author = "Your Name"
enabled = true

[commands]
command1 = "Description of command1"
command2 = "Description of command2"
```

### lib.rs

The `lib.rs` file is where you implement your card's functionality:

```rust
use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MyCard {
    name: String,
    version: String,
    description: String,
    config: CardConfig,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CardConfig {
    // Add any configuration options your card needs
    pub option1: String,
    pub option2: bool,
}

impl Card for MyCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn initialize(&mut self, config: &CardConfig) -> Result<()> {
        // Initialize your card with the provided configuration
        self.config = config.clone();
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "command1" => {
                // Implement command1
                println!("Executing command1 with args: {:?}", args);
                Ok(())
            },
            "command2" => {
                // Implement command2
                println!("Executing command2 with args: {:?}", args);
                Ok(())
            },
            _ => bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "command1".to_string(),
                description: "Description of command1".to_string(),
                usage: "pocket cards run my-card command1 [args...]".to_string(),
            },
            CardCommand {
                name: "command2".to_string(),
                description: "Description of command2".to_string(),
                usage: "pocket cards run my-card command2 [args...]".to_string(),
            },
        ]
    }
    
    fn cleanup(&self) -> Result<()> {
        // Perform any cleanup when the card is unloaded
        Ok(())
    }
}

// This is the entry point for the dynamic library
#[no_mangle]
pub extern "C" fn create_card() -> Box<dyn Card> {
    Box::new(MyCard {
        name: "my-card".to_string(),
        version: "0.1.0".to_string(),
        description: "Description of my card".to_string(),
        config: CardConfig::default(),
    })
}
```

## Building Your Card

Once you've implemented your card, you can build it using:

```bash
pocket cards build my-card
```

This will compile your card and place the dynamic library in the appropriate location for Pocket to find it.

## Testing Your Card

After building, you can test your card:

```bash
# List available cards to verify yours is there
pocket cards list

# Enable your card if it's not already enabled
pocket cards enable my-card

# Run one of your commands
pocket cards run my-card command1
```

## Advanced Topics

### Accessing Pocket's API

Your card can access Pocket's API by adding the `pocket-cli` crate as a dependency:

First, add the package
```bash
cargo add pocket-cli
```
And then, add it to your `Cargo.toml`
```toml
[dependencies]
pocket-cli = { path = "/path/to/pocket" }
```

This allows you to interact with Pocket's core functionality, such as:

- Accessing the storage manager to read/write snippets
- Using the version control system
- Interacting with other cards

> NOTE: if you run into any issues please reach out.

### Configuration Management

Cards can have their own configuration options. These are stored in the `card.toml` file and can be accessed through the `CardConfig` struct.

To add custom configuration options:

1. Define them in your `CardConfig` struct
2. Access them in your card's methods
3. Users can modify them by editing the `~/.pocket/cards/my-card.toml` file

### Command Line Argument Parsing

For more complex command line argument parsing, you can use the `clap` crate:

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
```

Then in your `execute` method:

```rust
fn execute(&self, command: &str, args: &[String]) -> Result<()> {
    match command {
        "complex-command" => {
            use clap::Parser;
            
            #[derive(Parser)]
            struct Args {
                #[arg(short, long)]
                option: String,
                
                #[arg(short, long)]
                flag: bool,
                
                input: Vec<String>,
            }
            
            let args = Args::parse_from(std::iter::once("complex-command".to_string()).chain(args.iter().cloned()));
            
            // Use args.option, args.flag, args.input
            
            Ok(())
        },
        // ...
    }
}
```

## Examples

### Hello World Card

```rust
use anyhow::{Result, bail};

pub struct HelloWorldCard {
    name: String,
    version: String,
    description: String,
}

impl Card for HelloWorldCard {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn initialize(&mut self, _config: &CardConfig) -> Result<()> {
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "hello" => {
                let name = args.get(0).map(|s| s.as_str()).unwrap_or("World");
                println!("Hello, {}!", name);
                Ok(())
            },
            _ => bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "hello".to_string(),
                description: "Greet someone".to_string(),
                usage: "pocket cards run hello-world hello [name]".to_string(),
            },
        ]
    }
    
    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn create_card() -> Box<dyn Card> {
    Box::new(HelloWorldCard {
        name: "hello-world".to_string(),
        version: "0.1.0".to_string(),
        description: "A simple hello world card".to_string(),
    })
}
```

### Snippet Counter Card

This card counts the number of snippets in your Pocket:

```rust
use anyhow::{Result, bail};
use pocket_cli::storage::StorageManager;

pub struct CounterCard {
    name: String,
    version: String,
    description: String,
}

impl Card for CounterCard {
    // ... other trait methods ...
    
    fn execute(&self, command: &str, _args: &[String]) -> Result<()> {
        match command {
            "count" => {
                let storage = StorageManager::new(None)?;
                let entries = storage.list_entries(None)?;
                println!("You have {} snippets in your Pocket.", entries.len());
                Ok(())
            },
            _ => bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "count".to_string(),
                description: "Count the number of snippets".to_string(),
                usage: "pocket cards run counter count".to_string(),
            },
        ]
    }
}

#[no_mangle]
pub extern "C" fn create_card() -> Box<dyn Card> {
    Box::new(CounterCard {
        name: "counter".to_string(),
        version: "0.1.0".to_string(),
        description: "Count snippets in your Pocket".to_string(),
    })
}
```

## Troubleshooting

### Card Not Loading

If your card isn't showing up in `pocket cards list`:

1. Check that it's built correctly: `pocket cards build my-card`
2. Verify the dynamic library exists in `~/.pocket/wallet/my-card/target/release/`
3. Check for any error messages in the console output
4. Make sure the `create_card` function is exported with `#[no_mangle]`

### Runtime Errors

If your card is loading but commands are failing:

1. Add debug prints to your code to trace execution
2. Check that your card's configuration is correct
3. Verify that any dependencies your card needs are available

## Best Practices

1. **Keep it focused**: Cards should do one thing well
2. **Handle errors gracefully**: Use proper error handling with `anyhow::Result`
3. **Document your commands**: Provide clear descriptions and usage examples
4. **Version your card**: Follow semantic versioning for your card
5. **Test thoroughly**: Test your card with different inputs and edge cases

## Publishing Your Card

To share your card with others:

1. Push your card to a GitHub repository
2. Others can install it using:
   ```bash
   pocket cards add my-card https://github.com/username/pocket-card-my-card
   pocket cards build my-card
   ```

## Resources

- [Rust Documentation](https://doc.rust-lang.org/book/)
- [anyhow Crate](https://docs.rs/anyhow/latest/anyhow/)
- [clap Crate](https://docs.rs/clap/latest/clap/)
- [Pocket CLI Repository](https://github.com/frgmt0/pocket) 