# 🦘 Pocket

**Your code snippets deserve better than being scattered across random `useful-stuff.txt` files.**

## The Story of Pocket

It starts the same way for all of us. You write a particularly clever piece of code—perhaps a regex that validates email addresses while simultaneously making coffee, or a beautiful little function that transforms your data in just the right way. You smile, commit it, and move on with your life.

Three months later, you need that exact same solution again. Was it in the authentication service? That utility file in your personal project? Maybe you pasted it into a Slack message to yourself? You vaguely remember it was brilliant, but its location has vanished into the digital ether, forcing you to rewrite it from scratch.

This cycle repeats indefinitely, stealing precious minutes of your life that could be better spent arguing about tabs versus spaces or explaining to your non-technical friends why the printer isn't working (despite having nothing to do with printers professionally).

Pocket was born from this universal developer frustration—the knowledge that you've solved this exact problem before, coupled with the maddening inability to find that solution when you need it again.

## What Pocket Actually Is (Beyond the Existential Story)

Pocket is a command-line tool that serves as your personal code knowledge base. It lets you save snippets, templates, configurations, and any text-based content in a way that matches how your brain actually works. Instead of organizing by rigid folder structures or memorizing exact file paths, Pocket lets you retrieve content based on context, fuzzy recollections, and the general vibes of what you're looking for.

Think of it as that one friend who somehow remembers everything you've ever told them, but for your code. It's the digital equivalent of having a personal librarian dedicated to cataloging and retrieving all those brilliant solutions you've crafted over your career.

## Why Current Solutions Fall Short

We've all tried to solve this problem before. You might have a GitHub Gist collection that started organized but now resembles a digital junk drawer, with cryptic titles like "thing-i-need" and "important-dont-delete". Perhaps you're a Slack self-messenger, scrolling endlessly through your own monologues hoping to spot that configuration block among the lunch plans and reminders.

Some developers maintain elaborate systems of text files with naming conventions only they understand, while others leave trails of commented-out code throughout their projects like breadcrumbs they hope to follow home someday. The truly optimistic among us just trust their command history to preserve that perfect incantation forever.

All of these approaches start with good intentions but inevitably collapse under the weight of time and volume. They require too much discipline to maintain and too much clairvoyance to search effectively.

## How Pocket Changes the Game

Pocket approaches the problem differently. It doesn't expect you to remember exact details or maintain perfect organization. Instead, it adapts to the way developers naturally think about their code.

When you find a solution worth keeping, adding it to Pocket takes seconds:

```bash
# Add from a file
pocket add path/to/brilliant/solution.js

# Add directly from your thoughts
pocket add -m "SELECT * FROM users WHERE sanity > 0;"

# Open your editor when inspiration strikes
pocket add -e
```

Later, when you need to find that solution, you don't need to remember exactly what you called it or where you put it. You can search with natural language, describing the problem it solves rather than its exact syntax:

```bash
# Search with natural language
pocket search "that pagination thing with the cursor"
```

Pocket understands what you mean, not just what you type, using semantic search to find content based on concepts and similarity, not just exact keyword matches.

## The Organizational Philosophy

We humans aren't naturally organized creatures. We aspire to perfect systems but rarely maintain them. Pocket acknowledges this reality and offers a flexible approach called "backpacks." Think of backpacks as loose collections of related items—more forgiving than folders, more useful than tags.

When you find yourself accumulating snippets in a particular domain, you might create a backpack for them:

```bash
# Create a backpack for your React snippets
pocket create backpack react

# Add something directly to a backpack
pocket add -b react ComponentTemplate.jsx
```

This gives just enough structure to be useful without becoming a burden to maintain. Your future self will be impressed but not overwhelmed by your organizational prowess.

## The Workflow Revolution

Pocket truly shines in its ability to integrate with your actual workflow. Finding a snippet is useful, but inserting it directly where you need it is magical:

```bash
# Insert that perfect snippet right where you need it
pocket insert config.yaml
```

Pocket guides you through selecting the right content and placing it exactly where it belongs. But it doesn't stop there—it also allows you to create and run workflows in two powerful ways:

### 1. Direct Command Chains

You can chain commands together in a single line:

```bash
# Create and save a workflow directly
pocket lint "search 'auth middleware' > !(confirm then insert path/to/app.js) > save auth-flow"
```

### 2. Workflow Files

For more complex or reusable workflows, you can create `.pocket` files in `~/.pocket/data/workflows/`. These files allow you to:
- Document your workflows with comments
- Split complex workflows into multiple steps
- Share workflows with your team through version control

Here's an example workflow file (`~/.pocket/data/workflows/start-rust.pocket`):
```bash
# Workflow for starting a new Rust project
# Each line is executed as a command chain

# First, search for Rust project setup snippets
search "cargo new project setup"

# Then search for common Rust dependencies
search "rust common dependencies Cargo.toml"

# Look for error handling patterns
search "rust error handling anyhow"
```

Run your workflow by name (without the .pocket extension):
```bash
pocket lint start-rust
```

List all available workflows:
```bash
pocket lint
```

This will show all your workflows with their descriptions (taken from the first comment in each file).

These workflows transform what would be multiple commands into a single, repeatable action. They're like tiny programs within your command line, specific to your personal code knowledge.

## Real-World Developer Sagas (Where Pocket Saves the Day)

### The Config Odyssey: A Tale of Digital Archaeology

Every developer knows the pain of starting a new project. The coding is the fun part—it's the configuration that drains your soul faster than a vampire at a blood bank. You've copied the same ESLint, Prettier, and TypeScript configs across projects so many times that you're starting to dream in JSON. Each time with slight variations that inevitably cause errors so mysterious they could star in their own true crime podcast.

Enter Pocket, the Indiana Jones of your code archaeology expeditions. Save your battle-tested configurations with notes about what each option does and why you chose it (including that one weird hack that makes your testing framework stop crying). Now, starting a new project is as simple as:

```bash
pocket search "eslint typescript config" | pocket insert
```

The right configuration appears like magic—or like someone actually documented their code properly for once. Complete with comments explaining past decisions made by Past You (who was clearly smarter than Present You, somehow). You just saved yourself from the special kind of existential crisis that comes from staring at configuration files for three hours.

### The Algorithm Amnesia: When Your Brain's Cache Keeps Missing

You've implemented pagination logic across different services so many times that you should probably list "Professional Paginator" on your LinkedIn. Some use cursors, some use page numbers, some use both, and one uses an arcane system based on moon phases that seemed like a good idea at 3 AM. You *know* you wrote the perfect version somewhere, but was it in that microservice? That API? That dream you had after eating spicy food?

After adding your various pagination implementations to Pocket, finding your magnum opus is as easy as:

```bash
pocket search "pagination cursor-based"
```

Now when building a new service, you start from your best implementation rather than from scratch. Your code is finally consistent across projects, and your team stops making that face when they review your PRs—you know the one. It's like having a time machine that lets you retrieve your most brilliant thoughts without also retrieving the questionable fashion choices that accompanied them.

### The Boilerplate Breakthrough: Escaping Copy-Paste Purgatory

Your team has a specific pattern for React components that includes more boilerplate than a Victorian novel. It's got prop validation, default exports, connected test files, and enough structural conventions to make an architect jealous. New team members miss elements of this pattern so often that your code reviews now include a bingo card for spotting the mistakes.

By adding the complete pattern to Pocket, you've created an escape hatch from copy-paste purgatory:

```bash
pocket insert -b react component-template.jsx > src/components/NewFeature.jsx
```

New developers can instantly generate components that follow team standards, reducing the time spent on boilerplate by approximately 97.3% (a completely made-up but emotionally accurate statistic). They can focus on implementing features instead of memorizing structural conventions, and you can finally stop having nightmares about missing PropTypes declarations.

## Getting Pocket Into Your Life

Installing Pocket is as straightforward as the tool itself:

```bash
# Install with Cargo
cargo install pocket-cli

# Or download the binary for your platform
curl -sSL https://get.pocket-cli.dev | sh
```

Your journey with Pocket begins with a simple save:

```bash
# Add a snippet
pocket add -m "console.log('Hello Pocket!');"

# Create a workflow file
echo '# My first workflow
search "hello world"' > ~/.pocket/data/workflows/hello.pocket

# Run the workflow
pocket lint hello
```

From these first steps, you'll build a personal knowledge base that grows with your career, preserving all those brilliant solutions that previously vanished into the digital void.

## Command Reference

Here are the main commands you'll use with Pocket:

```bash
# Adding content
pocket add [FILE]                    # Add from a file
pocket add -m "content"             # Add directly
pocket add -e                       # Open in your editor
pocket add -b backpack-name        # Add to a specific backpack

# Finding content
pocket search "query"              # Search your snippets
pocket list                        # List all entries
pocket list --include-backpacks    # List entries in all backpacks

# Using content
pocket insert [ID] [FILE]          # Insert a snippet into a file
pocket remove [ID]                 # Remove a snippet

# Organization
pocket create backpack NAME        # Create a new backpack
pocket list --backpack NAME        # List entries in a backpack

# Workflows
pocket lint "command > command"    # Run a command chain
pocket lint workflow-name          # Run a .pocket workflow file
pocket lint                        # List available workflows

# Other
pocket version                     # Show version information
```

## Directory Structure

Pocket organizes your data in `~/.pocket/`:
```
~/.pocket/
  data/
    entries/              # Individual snippets
    backpacks/           # Organized collections
    workflows/           # Your .pocket workflow files
  config.toml           # Your configuration
```

## Versioning

Pocket uses a letter-based versioning system that prioritizes communication over artificial constraints. This approach makes it immediately clear what stage of development the software is in and how it relates to previous versions.

Our version format is: `v-pocket-XN[-nc]`

Where:
- `X` is a letter indicating stability:
  - `A` = Alpha (experimental, seeking feedback)
  - `B` = Beta (still buggy but usable for testing)
  - `C` = Candidate (almost ready for official release)
  - `R` = Release (stable and ready for production)
- `N` is a number indicating the iteration
- `-nc` (optional) indicates compatibility issues with previous versions

For example:
- `v-pocket-A1`: First alpha release
- `v-pocket-R2-nc`: Second stable release, not compatible with previous versions
- `v-pocket-R3-nc1<`: Third stable release, only compatible with version 1 and newer

Current version: `v-pocket-R1` (Workflow Files)

You can view detailed version information with:
```bash
pocket version
```

For a complete history of changes, see the [CHANGELOG.md](CHANGELOG.md) file.

## The Pocket Philosophy

At its core, Pocket is built on a simple belief: your code snippets are valuable intellectual assets that deserve better than being lost to time and disorganization. The hours you've spent perfecting regex patterns, crafting efficient algorithms, and fine-tuning configurations represent significant investments of your expertise and creativity.

Pocket treats these assets with the respect they deserve, making them retrievable not just tomorrow or next week, but years into your career. It acknowledges that finding should always be faster than rewriting, no matter how simple the code might seem. Most importantly, it adapts to the way you naturally think and work, rather than forcing you into rigid organizational systems that feel like a second job to maintain.

This philosophy extends to the tool itself, which remains unobtrusive, lightweight, and adaptable. Pocket doesn't demand cloud accounts, doesn't require internet access, and doesn't insist on a particular workflow. It sits quietly in your command line, ready to save and retrieve your knowledge exactly when you need it.

## Join the Pocket Community

Pocket is open source because knowledge management should be a communal effort. The challenges of code reuse and discovery are universal across programming languages, project types, and developer experiences. By contributing to Pocket, you're helping build a tool that respects the collective intelligence of developers everywhere.

Whether you're interested in improving the semantic search capabilities, building new extensions, or simply sharing your own use cases, your experience enriches the project. Check our [CONTRIBUTING.md](CONTRIBUTING.md) to learn how to get involved.

## License

Pocket is available under the MIT License—because your code snippets have suffered enough without adding licensing anxiety to the mix.

---

*Pocket: Because life's too short to write the same code twice.*