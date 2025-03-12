# Cards System

Cards are Pocket's plugin system, allowing you to extend the functionality of the CLI with custom commands and features.

## Using Cards

### Listing Available Cards

```bash
pocket cards list
```

This will show all installed cards and their status (enabled/disabled).

### Running Card Commands

```bash
pocket cards run card_name command [args...]
```

For example, to run the "backup" command from the backup card:

```bash
pocket cards run backup backup
```

### Enabling and Disabling Cards

```bash
# Enable a card
pocket cards enable card_name

# Disable a card
pocket cards disable card_name
```

## Installing Cards

### From GitHub

```bash
pocket cards add card_name https://github.com/username/pocket-card-name
pocket cards build card_name
```

### Creating a Local Card

```bash
pocket cards create card_name "Description of the card"
```

This will create a new card in your wallet directory (`~/.pocket/wallet/card_name`).

After creating the card, you'll need to implement your functionality in the `src/lib.rs` file and then build it:

```bash
pocket cards build card_name
```

## Developing Cards

Cards are Rust libraries that implement the `Card` trait. The basic structure is:

```rust
pub struct Card {
    name: String,
    version: String,
    description: String,
    config: CardConfig,
}

impl Card {
    // Implementation of card functionality
    
    pub fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "command_name" => {
                // Command implementation
                Ok(())
            },
            _ => anyhow::bail!("Unknown command: {}", command),
        }
    }
    
    pub fn commands(&self) -> Vec<CardCommand> {
        // Return a list of available commands
    }
}
```

### Card Directory Structure

```
card_name/
├── Cargo.toml
├── card.toml
├── README.md
└── src/
    └── lib.rs
```

For more detailed information on developing cards, see the [Card Development Guide](https://github.com/frgmt0/pocket/wiki/Card-Development-Guide). 