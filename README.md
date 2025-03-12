# 🦘 Pocket

CLI tool for devs who can't find their stuff.

## What Even Is This Thing

Pocket is a CLI tool that saves your code snippets and lets you find them later. You write code, it's good, you save it, you need it again later.

Sometimes I think about how much time we waste looking for stuff we already wrote. It's like... probably at least 3 hours a week? Maybe more? Who knows. Anyway.

## Testing the Alpha/Beta Version

To test the latest beta version with VCS functionality, you'll need to compile from source:

```bash
# Clone the repository with the experimental branch
git clone -b experimental/version-control-with-pocket https://github.com/frgmt0/pocket-cli.git
cd pocket-cli

# Build the release version
cargo build --release

# Run commands using the local binary
./target/release/pocket status
./target/release/pocket pile src/*
./target/release/pocket shove -m "My first commit"
```

The version control commands are still in alpha, so please report any issues you encounter so I can make it better.

## Commands
This is the majority of your work here, and its pretty easy to grasp.

```bash
# Add stuff
pocket add file.js                      # from a file
pocket add -m "code goes here"          # inline
pocket add -e                           # opens your editor (coming soon)

# Find stuff
pocket search "that thing with the loop"
pocket list

# Use stuff
pocket insert [ID] [file]               # put snippet in file (else show some kind of tui or whatever)
pocket remove [ID]                      # delete forever

# Organize stuff
pocket create backpack name             # group related things
pocket add -b backpack file.js          # add to backpack

# Chain commands
pocket lint > search "auth" > insert file.js
```

## Backpacks

Folders are annoying. Backpacks are better. but pocket basically uses these folders to like manage your entries regarding a specific topic. this can help sometimes with more narrow scope implementations

```bash
pocket create backpack rust
pocket add -b rust my_awesome_code.rs
```

Then later when you're doing that Rust thing again:

```bash
pocket search "that rust thing with the macros and stuff"
```

it works for the most part but i think later we may move to a small kind of llm like BERT or something to actualy handle searches better, but for now this works.

## Workflows

So the REALLY cool part is you can chain commands together. Like for example say you always forget how to set up a new React project you can do:

```bash
pocket lint "search 'react setup' > insert ./" # this would ideally call some bash script but i need to maybe get some kind of auto-run going later
```

Or save workflows in files if you want, I guess? Put them in `~/.pocket/data/workflows/whatever.pocket` and run them with `pocket lint whatever`.

## Real Use Cases

### Config Hell

You know when you start a new project and need to copy all those config files over but you can't remember which project had the good ones? And then you end up with like 15 different ESLint configs and none of them work?

Just do:
```bash
pocket search "eslint" | pocket insert
```

### Algorithm Amnesia

Wait did I already solve this exact problem before? Yes, yes you did. Last month. And the month before. And probably next month too.

```bash
pocket search "pagination thing"
```

## Install

```bash
cargo install pocket-cli
```

I mean you could build from source too but why would you do that to yourself?

## Directory Structure

All your stuff goes in `~/.pocket/`

## Version ??

We have letters instead of semantic versioning because honestly who even knows what 2.1.3-alpha.7+metadata.12 means. Our versions look like:

- `v-pocket-A1` = alpha, probably broken
- `v-pocket-R2` = release 2, mostly works

Current: `v-pocket-R3A2-ncR3A1<`

you can read about my ideas on version [here](https://blog.frgmt.xyz/03102025-tech)

## License

MIT