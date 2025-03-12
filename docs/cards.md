# Cards System: Pocket's Plugin Party

Cards are basically Pocket's way of letting you bolt on whatever cool functionality you want. Think of them as plugins that extend what the CLI can do without bloating the core.

## Using Cards (The Basics)

### Finding What You've Got

```bash
pocket cards list
```

This shows you all the cards you've installed and whether they're actually turned on. Because having a bunch of disabled cards is just digital hoarding.

### Making Cards Do Things

```bash
pocket cards run card_name command [args...]
```

For example, if you want the backup card to, you know, actually back up your stuff:

```bash
pocket cards run backup backup
```

Yeah, it's "backup backup" - we're not winning any UX awards here, but it works.

### Turning Cards On and Off

```bash
# Turn a card on
pocket cards enable card_name

# Tell a card to take a nap
pocket cards disable card_name
```

## Getting New Cards

### Grab One From GitHub

```bash
pocket cards add card_name https://github.com/username/pocket-card-name
pocket cards build card_name
```

Just a heads up - you're running someone else's code. Trust accordingly.

### DIY: Make Your Own Card

```bash
pocket cards create card_name "Does something awesome probably"
```

This creates a skeleton card in `~/.pocket/wallet/card_name`. It won't do anything useful yet.

After adding your actual code to `src/lib.rs`, build it with:

```bash
pocket cards build card_name
```

## Card Development (The Fun Part)

Cards are Rust libraries implementing the `Card` trait. The TL;DR version looks like:

```rust
pub struct MyAwesomeCard {
    name: String,
    version: String,
    description: String,
    config: CardConfig,
}

impl Card for MyAwesomeCard {
    // getters and boring stuff omitted
    
    fn execute(&self, command: &str, args: &[String]) -> Result<()> {
        match command {
            "do_cool_thing" => {
                // Your actually useful code goes here
                println!("Did the cool thing with: {:?}", args);
                Ok(())
            },
            _ => anyhow::bail!("Unknown command: {}", command),
        }
    }
    
    fn commands(&self) -> Vec<CardCommand> {
        // Tell Pocket what commands your card supports
        vec![
            CardCommand {
                name: "do_cool_thing".to_string(),
                description: "Does that cool thing you wanted".to_string(),
                usage: "pocket cards run my-card do_cool_thing [stuff]".to_string(),
            }
        ]
    }
}
```

### What's In The Box

When you create a card, you get:

```
your_card/
├── Cargo.toml      # dependencies and metadata
├── card.toml       # card config 
├── README.md       # where your docs should go but probably won't
└── src/
    └── lib.rs      # where the magic happens
```

For a deeper dive into card development with all the nerdy details, check out the [Card Development Guide](card-development-guide.md). It's like this guide but with more code snippets and fewer attempts at my humor.