# Contributing to Pocket
*The contributor guide that doesn't make you want to close the tab immediately*

## Welcome to the Pocket Community

First off, the fact that you're reading a CONTRIBUTING.md file means you're already in the top 1% of humans who might actually help improve this project instead of just complaining about it on Twitter. For that alone, we're grateful.

Contributing to Pocket isn't just about writing code that works—it's about creating a tool that makes developers' lives less frustrating. Because let's be honest, we all have enough existential dread without also having to rewrite the same RegEx pattern for the fifth time this year.

## Design Philosophy

Pocket is built around several core design principles that should guide all contributions:

### 1. Simplicity Over Complexity

Pocket should be intuitive and easy to use, even for developers who are new to the tool. This means:
- Commands should follow predictable patterns
- Features should be discoverable
- Documentation should be clear and accessible
- Complex functionality should have sensible defaults

### 2. Adaptability Over Rigidity

Pocket adapts to how developers actually work, not the other way around:
- Support various workflows without forcing a specific one
- Allow for different organizational styles
- Provide flexibility in how snippets are categorized and retrieved
- Enable customization through the Cards system

### 3. Utility Over Novelty

Every feature should solve a real problem that developers face:
- Focus on practical use cases over theoretical ones
- Prioritize features that save time and reduce cognitive load
- Avoid adding features just because they're technically interesting
- Consider the long-term maintenance cost of new features

### 4. Consistency in Design

Pocket maintains a consistent design language throughout:
- Terminology is consistent across features (e.g., "Cards" for plugins, "Backpacks" for collections)
- Command structure follows predictable patterns
- Error messages are helpful and actionable
- Visual styling (colors, formatting) is consistent in CLI output

### 5. Progressive Disclosure

Simple operations should be simple, while advanced features are available when needed:
- Basic commands should work with minimal arguments
- Advanced options should be discoverable but not required
- Help text should guide users to related commands
- Complex workflows can be automated through the workflow system

### 6. Thoughtful Defaults

Pocket should work well out of the box:
- Sensible default configurations
- Automatic organization where possible
- Smart search that understands developer intent
- Minimal setup required for basic usage

## The Philosophy Behind Pocket

Pocket exists because developers shouldn't have to choose between maintaining an elaborate organizational system that requires more upkeep than a vintage car or randomly scattering useful code across Slack messages, GitHub gists, and hastily-named text files.

When contributing to Pocket, remember we're building a tool that adapts to how developers actually work—chaotic, context-switching, and perpetually convinced they'll "definitely remember" where they put that brilliant piece of code (narrator: they won't).

## Ways to Contribute (All Equally Valuable)

### Code Contributions

Yes, code is the obvious one, but before you start refactoring everything into a single line of undecipherable Rust, consider:

- **Bug fixes**: These are the unsung heroes of open source. Found something broken? Fix it. We promise not to ask why you were using Pocket at 3 AM.

- **Features**: Have an idea that would make Pocket even better at solving the "where did I put that code" problem? Implement it. Just make sure it aligns with our core philosophy of "making it easier to find your stuff without requiring you to be unrealistically organized."

- **Performance improvements**: If you can make Pocket faster without requiring users to sacrifice their firstborn to the Rust compiler, we're interested.

### Documentation

Documentation is like dental floss—everyone knows they should use it more, but nobody wants to do it. That's why documentation contributions are worth their weight in gold (or Bitcoin, if that's still a thing when you're reading this).

Good documentation includes:
- Clear explanations that don't assume the reader has a PhD in Computer Science
- Examples that actually work and aren't just theoretical
- A tone that acknowledges the reader is a human being with limited patience and many browser tabs open

### Bug Reports

Found a bug but don't have time to fix it? No problem. When submitting a bug report, try to include:

- What you were doing when the bug occurred (wrong answers only... kidding, please be accurate)
- What you expected to happen
- What actually happened instead
- Your environment details (OS, Pocket version, whether Mercury is in retrograde)

A good bug report is already halfway to being fixed. A bad bug report is just "it doesn't work" followed by tumbleweeds.

### Feature Requests

Have an idea but not the time or skills to implement it? Share it anyway. The best feature requests:

- Clearly describe the problem you're trying to solve
- Explain why existing features don't quite cut it
- Don't require us to rewrite the entire codebase or violate the laws of physics

## The Technical Stuff

### Setting Up Your Development Environment

1. **Fork & Clone**: Standard GitHub stuff. Fork the repo, clone it locally.
2. **Install Dependencies**: Run `cargo build` and pray to the dependency gods.
3. **Run Tests**: Execute `cargo test` and feel that brief moment of joy when everything passes.

### Code Style & Standards

We use a code style that can best be described as "readable by humans who haven't slept enough." More specifically:

- Run `cargo fmt` before committing. It's automated style enforcement so we don't have to argue about formatting.
- Use meaningful variable names. Future you will thank present you when debugging at midnight.
- Comments should explain why, not what. The code shows what; good comments explain the reasoning that isn't obvious from reading the code.

### The Pull Request Process

1. **Branch Naming**: Use a descriptive name like `feature/amazing-new-thing` or `fix/thing-that-was-horribly-broken`.
2. **Keep PRs Focused**: One feature or fix per PR. We're not looking for a PR that solves all of software engineering's problems at once.
3. **Tests**: Add tests for your changes. Untested code is like an unchaperoned teenager—likely to cause problems when you're not looking.
4. **Documentation**: Update docs to reflect your changes. Yes, all of them.
5. **PR Description**: Explain what your changes do and why they should be merged. "It works on my machine" is not sufficient justification.

### Commit Messages

Good commit messages are like good documentation—rare and precious. We follow these guidelines:

- Start with a verb in the present tense: "Add feature" not "Added feature"
- Keep the first line under 72 characters
- Use the body to explain what and why, not how

For example:
```
Add support for quantum encryption backpacks

This implements a new backpack type that uses quantum encryption to
secure extremely sensitive snippets. Users working with government
secrets or their Netflix password can now store them safely.

Fixes #42
```

## The Human Stuff

### Code of Conduct

We have one rule: Don't be a jerk. This covers most situations, but if you need clarification:

- Treat others with respect and empathy
- Assume good intentions, even when someone's code makes you question their sanity
- Remember that text-based communication lacks tone—that person isn't attacking you, they just forgot to add emojis
- Give constructive feedback, not just criticism

### The Review Process

Your PR will be reviewed by maintainers who are balancing this open source project with day jobs, families, and existential crises about their career choices. Please be patient.

When receiving feedback:
- Don't take it personally
- Ask questions if something isn't clear
- Remember that every suggestion is aimed at making the project better, not criticizing your abilities

## Card Development

Pocket's Card system (formerly known as plugins) allows for extending functionality. When developing Cards:

1. **Do One Thing Well**: The Unix philosophy never goes out of style
2. **Document Extensively**: Assume users haven't read any other documentation
3. **Error Handling**: Users will find ways to break your Card you never imagined
4. **Dependencies**: Keep them minimal and well-justified

For detailed information on developing Cards, see the [Card Development Guide](docs/card-development-guide.md).

## Version Control System

Pocket includes a built-in version control system. When contributing to this system:

1. **Keep It Simple**: The VCS should be intuitive, especially for users who find Git confusing
2. **Consistent Terminology**: Use the established terms (Pile, Shove, Timeline) consistently
3. **Error Messages**: Provide clear guidance when something goes wrong
4. **Performance**: Consider the performance implications of VCS operations

For more details, see the [Version Control System documentation](docs/version-control.md).

## Project Structure

```
pocket/
├── .github/            # GitHub configuration files
├── docs/               # Documentation
│   ├── cards.md        # Cards system documentation
│   ├── card-development-guide.md # Guide for developing Cards
│   ├── commands.md     # Command reference
│   ├── installation.md # Installation guide
│   └── version-control.md # Version control system documentation
├── src/                # Source code
│   ├── cards/          # Cards (plugin) system
│   ├── commands/       # Command implementations
│   ├── models/         # Data models
│   ├── search/         # Search functionality
│   ├── storage/        # Storage management
│   ├── utils/          # Utility functions
│   ├── vcs/            # Version control system
│   ├── version.rs      # Version information
│   └── main.rs         # Entry point
├── Cargo.toml          # Project configuration
└── README.md           # Project documentation
```

## Versioning

Pocket uses a combination of semantic versioning and a letter-based system:

- SemVer format: `0.6.2` (for Cargo and the Rust ecosystem)
- Letter format: `v-pocket-R3B1-ncR2<` (for human communication)
- Date format: `03252025` (for internal tracking)

When making changes:
- Increment the patch version for bug fixes
- Increment the minor version for new features
- Increment the major version for breaking changes
- Update the letter-based version and date in `src/version.rs`

## In Conclusion

Contributing to open source is a strange mix of altruism, resume-building, and scratching your own itch. Whatever your motivation, we're glad you're here.

Remember, every contribution—from a typo fix to a major feature—helps make Pocket better for developers everywhere. Each time you contribute, you're saving someone from the special kind of rage that comes from knowing they've solved this exact problem before but can't find the code.

And that, friends, is a genuine service to humanity.

---

*Now go build something awesome. Preferably using Pocket to store all the brilliant code you'll write along the way.*

Thank you for considering contributing to Pocket! This document outlines the process for contributing to the project and how to get started.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## How Can I Contribute?

### Reporting Bugs

Bugs are tracked as GitHub issues. Create an issue and provide the following information:

- Use a clear and descriptive title
- Describe the exact steps to reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed after following the steps
- Explain which behavior you expected to see instead and why
- Include screenshots if possible

### Suggesting Enhancements

Enhancement suggestions are also tracked as GitHub issues. When creating an enhancement suggestion:

- Use a clear and descriptive title
- Provide a detailed description of the suggested enhancement
- Explain why this enhancement would be useful to most Pocket users
- Provide examples of how it would be used

### Pull Requests

- Fill in the required template
- Follow the Rust style guide
- Include tests for new features or bug fixes
- Update documentation as needed
- End all files with a newline

## Development Workflow

### Setting Up the Development Environment

1. Fork the repository
2. Clone your fork: `git clone https://github.com/frgmt0/pocket.git`
3. Add the upstream repository: `git remote add upstream https://github.com/frgmt0/pocket.git`
4. Install dependencies: `cargo build`

### Making Changes

1. Create a new branch: `git checkout -b your-branch-name`
2. Make your changes
3. Run tests: `cargo test`
4. Run the linter: `cargo clippy`
5. Format your code: `cargo fmt`
6. Commit your changes with a descriptive message
7. Push to your fork: `git push origin your-branch-name`
8. Create a pull request

### Versioning

Pocket uses a letter-based versioning system:

- Format: `v-pocket-XN[-nc]`
- `X` is a letter indicating stability:
  - `A` = Alpha (experimental, seeking feedback)
  - `B` = Beta (still buggy but usable for testing)
  - `C` = Candidate (almost ready for official release)
  - `R` = Release (stable and ready for production)
- `N` is a number indicating the iteration
- `-nc` (optional) indicates compatibility issues with previous versions

## Project Structure

```
pocket/
├── .github/            # GitHub configuration files
├── src/                # Source code
│   ├── commands/       # Command implementations
│   ├── models/         # Data models
│   ├── search/         # Search functionality
│   ├── storage/        # Storage management
│   ├── utils/          # Utility functions
│   ├── version.rs      # Version information
│   └── main.rs         # Entry point
├── Cargo.toml          # Project configuration
└── README.md           # Project documentation
```

## Testing

- Run all tests: `cargo test`
- Run specific tests: `cargo test test_name`
- Run with coverage: `cargo tarpaulin`

## Documentation

- Update the README.md for user-facing changes
- Document code with rustdoc comments
- Update CHANGELOG.md for new releases

## Questions?

If you have any questions, feel free to create an issue or reach out to me.

Thank you for contributing to Pocket!