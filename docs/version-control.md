# Pocket CLI Version Control System
*Where Git meets simplicity, and version history becomes less intimidating*

Pocket includes a built-in version control system that functions remarkably like Git, but with terminology that won't make you question your career choices. It's designed for those moments when you need version control but don't want to remember whether you should pull, fetch, or rebase.

> **⚠️ IMPORTANT DISCLAIMER ⚠️**  
> This version control system is currently in Beta. While we've designed it to be reliable, consider it the daring younger sibling to Git's established stability. We highly recommend keeping Git as your backup solution for mission-critical projects, at least until we've ironed out all the quirks. We welcome your feedback, but please use at your own risk for anything you can't afford to lose!

## Key Concepts: Version Control for Humans

- **Repository**: A collection of files and their history (just like Git, but we kept this term because it actually makes sense)
- **Pile**: The staging area where changes gather before commitment (Git calls this the "index" which sounds like something from a textbook)
- **Shove**: A commit or snapshot of your files (because you're literally shoving changes into history)
- **Timeline**: A branch or parallel universe of development (more intuitive than "branch," which evokes complicated tree metaphors)
- **Remote**: A connection to another repository (we kept this term too—because "that other computer with your code" was too long)

## Basic Workflow: Version Control Without the Anxiety

### Creating a Repository

```bash
pocket new-repo [path]
```

This initializes a new repository in the specified path (or current directory if you're too busy to type more words).

### Checking Status

```bash
pocket status
```

Shows the current state of your repository, including which files you've changed and which timeline you're currently manipulating history in.

### Adding Files to the Pile

```bash
# Add specific files
pocket pile file1.txt file2.js

# Add all files (the "I don't have time to decide" option)
pocket pile --all

# Add files matching a pattern
pocket pile --pattern "*.js"
```

### Creating a Shove (Commit)

```bash
# With a message
pocket shove -m "Fixed that bug that shouldn't have existed"

# Open editor for a more thoughtful message
pocket shove -e
```

### Viewing History

```bash
# Basic log (for the minimalists)
pocket log

# Detailed log (for the historians)
pocket log --verbose
```

## Working with Timelines: Parallel Universe Management

### Creating a Timeline

```bash
pocket timeline new timeline_name
```

Because "new-feature-that-might-break-everything" deserves its own universe.

### Switching Timelines

```bash
pocket timeline switch timeline_name
```

Universe hopping has never been this straightforward.

### Listing Timelines

```bash
pocket timeline list
```

See all the parallel universes you've created, and contemplate which one will eventually become your reality.

### Merging Timelines

```bash
pocket merge timeline_name
```

Collide universes together and hope for the best. (Unlike real universe collisions, this is generally safe.)

## Working with Remotes: Playing Nicely with Others

### Adding a Remote

```bash
pocket remote add remote_name url
```

Tell your local repository about other repositories it should be friends with.

### Listing Remotes

```bash
pocket remote list
```

Check which digital friends your repository is allowed to talk to.

### Fetching from a Remote

```bash
pocket fish [remote_name]
```

Yes, we used "fish" instead of "fetch" because fishing for changes sounds more rewarding than fetching them.

### Pushing to a Remote

```bash
pocket push [remote_name] [timeline_name]
```

Share your genius with the world, or at least with that specific other repository.

## Ignoring Files: The Digital Equivalent of "I Can't See You"

```bash
# Add a pattern to ignore
pocket ignore --pattern "*.log"

# Remove a pattern (forgiveness)
pocket ignore --remove "*.log"

# List ignore patterns (see who you're currently snubbing)
pocket ignore --list
```

## When Things Go Wrong

Remember, version control is essentially time travel for your code. And like all time travel scenarios in science fiction, occasionally things get weird. If you find yourself in an alternate timeline where nothing makes sense:

1. Take a deep breath
2. Run `pocket status` to see where you are
3. Consider if this is a good time to fall back to Git
4. Remember that even experienced developers sometimes delete everything and start over

## Final Thoughts

Our version control system aims to take the complexity out of tracking your code's evolution. While we've designed it to be intuitive, remember that it's still evolving itself. We're confident it will handle your day-to-day needs, but for those mission-critical projects or before major releases, consider a Git backup until we graduate from Beta.

Happy versioning, and may your code history always tell a story you're proud of!