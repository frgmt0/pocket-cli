# Card Development Guide: Extending Pocket CLI

So you wanna build your own cards for Pocket CLI? Sweet. This guide will walk you through the whole process without the usual documentation dryness. Let's get into it.

## WTF Are Cards Anyway?

Cards are basically Pocket's plugin system - they let you add your own custom commands and features to the CLI. Under the hood, they're just Rust libraries that implement the `Card` trait. Nothing too mysterious.

## Before You Start

You'll need:
- Rust + Cargo (v1.70.0+)
- Some basic Rust knowledge (you don't need to be a guru)
- Pocket CLI installed (duh)

## Creating Your First Card

### The Lazy Way (Recommended)

The easiest approach is just letting Pocket do the heavy lifting:

```bash
pocket cards create my-card "Does some cool stuff probably"
```

This creates a new card in `~/.pocket/wallet/my-card` with all the boilerplate stuff taken care of:

```
my-card/
├── Cargo.toml
├── card.toml
├── README.md
└── src/
    └── lib.rs
```

### The Manual Way (For Control Freaks)

If you prefer doing things by hand:

1. Create a directory in `~/.pocket/wallet/` with your card name
2. Set up all the files yourself like some kind of masochist

## The Important Files

### Cargo.toml

Your `Cargo.toml` should look something like:

```toml
[package]
name = "pocket-card-my-card"
version = "0.1.0"
edition = "2021"
description = "Does some cool stuff probably"
authors = ["You <your.email@example.com>"]
license = "MIT"

[lib]
name = "pocket_card_my_card"
crate-type = ["cdylib"]  # this part is crucial

[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Don't forget that `crate-type = ["cdylib"]` - it's essential for dynamic loading.

### card.toml

This is just some metadata about your card:

```toml
[card]
name = "my-card"
version = "0.1.0"
description = "Does some cool stuff probably"
author = "You"
enabled = true

[commands]
do_thing = "Makes the thing happen"
other_thing = "Does that other thing you wanted"
```

### lib.rs (The Important Bit)

Here's where the actual functionality lives:

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
    // stuff your card needs to know about
    pub some_option: String,
    pub another_flag: bool,
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
        // setup stuff goes here
        self.config = config.clone();
        Ok(())
    }
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "do_thing" => {
                // make the magic happen
                println!("Doing the thing with: {:?}", args);
                Ok(())
            },
            "other_thing" => {
                // do that other thing
                println!("Other thing happening with: {:?}", args);
                Ok(())
            },
            _ => bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "do_thing".to_string(),
                description: "Makes the thing happen".to_string(),
                usage: "pocket cards run my-card do_thing [stuff...]".to_string(),
            },
            CardCommand {
                name: "other_thing".to_string(),
                description: "Does that other thing".to_string(),
                usage: "pocket cards run my-card other_thing [args...]".to_string(),
            },
        ]
    }
    
    fn cleanup(&self) -> Result<()> {
        // any cleanup when your card gets unloaded
        Ok(())
    }
}

// this bit is essential - don't mess it up
#[no_mangle]
pub extern "C" fn create_card() -> Box<dyn Card> {
    Box::new(MyCard {
        name: "my-card".to_string(),
        version: "0.1.0".to_string(),
        description: "Does some cool stuff probably".to_string(),
        config: CardConfig::default(),
    })
}
```

## Building Your Card

Building is pretty straightforward:

```bash
pocket cards build my-card
```

This compiles your card and puts the dynamic library where Pocket can find it.

## Making Sure It Works

After building:

```bash
# check if your card shows up
pocket cards list

# enable it if needed
pocket cards enable my-card

# take it for a spin
pocket cards run my-card do_thing
```

## Next-Level Stuff

### Tapping into Pocket's Internals

Your card can use Pocket's API by adding the dependency:

```bash
cargo add pocket-cli
```

Then in your `Cargo.toml`:
```toml
[dependencies]
pocket-cli = "0.6.2"
```

This lets you do things like:
- Access storage to read/write snippets
- Use the version control system
- Talk to other cards

> if something breaks here, reach out. i'm still figuring some of this out.

### Config Management

Cards can have their own settings in the `card.toml` file.

To add custom options:
1. Define them in your `CardConfig` struct
2. Use them in your card's methods
3. Users can tweak them by editing `~/.pocket/cards/my-card.toml`

### Fancy Argument Parsing

For more complex CLI args, clap is your friend:

```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
```

Then in your code:

```rust
fn execute(&self, command: &str, args: &[String]) -> Result<()> {
    match command {
        "fancy-command" => {
            use clap::Parser;
            
            #[derive(Parser)]
            struct Args {
                #[arg(short, long)]
                some_option: String,
                
                #[arg(short, long)]
                some_flag: bool,
                
                stuff: Vec<String>,
            }
            
            let args = Args::parse_from(std::iter::once("fancy-command".to_string()).chain(args.iter().cloned()));
            
            // now you can use args.some_option, args.some_flag, args.stuff
            
            Ok(())
        },
        // ...
    }
}
```

## Example Cards

### Hello World (The Simplest Possible Card)

```rust
use anyhow::{Result, bail};

pub struct HelloWorldCard {
    name: String,
    version: String,
    description: String,
}

impl Card for HelloWorldCard {
    // basic getters omitted for brevity
    
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

### Snippet Counter (Slightly More Useful)

```rust
use anyhow::{Result, bail};
use pocket_cli::storage::StorageManager;

pub struct CounterCard {
    name: String,
    version: String,
    description: String,
}

impl Card for CounterCard {
    // other stuff omitted
    
    fn execute(&self, command: &str, _args: &[String]) -> Result<()> {
        match command {
            "count" => {
                let storage = StorageManager::new(None)?;
                let entries = storage.list_entries(None)?;
                println!("You've got {} snippets in your Pocket.", entries.len());
                Ok(())
            },
            _ => bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        vec![
            CardCommand {
                name: "count".to_string(),
                description: "Count your snippets".to_string(),
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
        description: "Count stuff in your Pocket".to_string(),
    })
}
```

## When Things Break

### Card Won't Load

If your card isn't showing up in `pocket cards list`:

1. Check that it built correctly: `pocket cards build my-card`
2. Make sure the library exists in `~/.pocket/wallet/my-card/target/release/`
3. Look for any errors in the console
4. Double-check that `create_card` function has `#[no_mangle]`

### Runtime Disasters

If your card loads but crashes:

1. Add some println debugging (the oldest trick in the book)
2. Check your card's config
3. Make sure all dependencies are where they should be

## Tips for Not Sucking

1. **Focus**: Cards should do one thing well, not twenty things poorly
2. **Error handling**: Use `anyhow::Result` instead of panicking
3. **Documentation**: Help your users understand your commands
4. **Versioning**: Follow semver so upgrades don't break stuff
5. **Testing**: Try to break your card before others do it for you

## Sharing Your Card

Want to inflict your creation on others?

1. Push it to GitHub
2. They can install it with:
   ```bash
   pocket cards add my-card https://github.com/yourusername/pocket-card-my-card
   pocket cards build my-card
   ```

## Useful Links

- [Rust Book](https://doc.rust-lang.org/book/) (for when you forget how borrowing works)
- [anyhow Docs](https://docs.rs/anyhow/latest/anyhow/) (for error handling)
- [clap Docs](https://docs.rs/clap/latest/clap/) (for fancy CLI stuff)
- [Pocket CLI Repo](https://github.com/frgmt0/pocket) (the mothership)