# Shell Hooks in Pocket CLI
*The document about extending your terminal you'll actually enjoy reading*

Shell hooks are where your terminal finally gets the personality upgrade it deserves. Pocket CLI offers two flavors of shell integrations through the `blend` command:

1. **Shell Extensions**: Scripts that integrate with your shell startup process, adding aliases, functions, and environment variables
2. **Executable Hooks**: Scripts you can run with the `@` prefix, simplifying your command workflow

## Overview

The `blend` command lets you:

1. **Install** shell scripts as either extensions or executable hooks
2. **Manage** these hooks with simple commands
3. **Edit** existing hooks when you want to update them
4. **Run** executable hooks directly

All hooks are stored in the `~/.pocket/hooks` directory, keeping your terminal extensions organized in one place.

## Basic Usage

### Adding a Shell Extension

```bash
pocket blend my_aliases.sh
```

This command:
1. Copies your script to `~/.pocket/hooks/my_aliases.sh`
2. Adds a line to your shell config file to source this script
3. Makes your aliases available after you restart your terminal or source your config file

### Adding an Executable Hook

```bash
pocket blend --executable my_script.sh
```

This command:
1. Copies your script to `~/.pocket/hooks/my_script.sh`
2. Makes it executable
3. Creates a wrapper script named `@my_script` in `~/.pocket/bin/`
4. Adds that bin directory to your PATH
5. Allows you to run the script by typing `@my_script` in your terminal

### Listing Hooks

```bash
pocket blend list
```

This displays all your installed hooks with their paths and types.

### Editing a Hook

```bash
pocket blend edit hook_name
```

Opens the hook in your default editor (using the `$EDITOR` environment variable).

### Running an Executable Hook

You can run an executable hook in two ways:

```bash
# Direct method
@hook_name [arguments]

# Using the blend command
pocket blend run hook_name [arguments]
```

## Shell Extensions Best Practices

For shell extensions, follow these guidelines:

1. **Always include a shebang line**:
   ```bash
   #!/bin/bash
   # or
   #!/bin/zsh
   ```
   This indicates the interpreter to use, even though it will be sourced and not executed directly.

2. **Add comments** explaining what your hook does:
   ```bash
   #!/bin/bash
   # Pocket CLI hook: Developer aliases
   # Provides shortcuts for common development tasks
   ```

3. **Group related functionality**:
   ```bash
   # Git aliases
   alias gs='git status'
   alias gc='git commit'
   
   # Docker aliases
   alias dc='docker-compose'
   alias dps='docker ps'
   ```

4. **Use prefixes for aliases** to avoid accidentally overriding system commands:
   ```bash
   # Prefixed with 'pk' for Pocket-related commands
   alias pk='pocket'
   alias pka='pocket add'
   alias pks='pocket search'
   ```

## Executable Hooks Best Practices

For executable hooks, follow these additional guidelines:

1. **Make your script robust with proper error handling**:
   ```bash
   #!/bin/bash
   set -e  # Exit on error

   # Function for proper error handling
   handle_error() {
     echo "Error: $1" >&2
     exit 1
   }
   
   # Check for required tools
   command -v docker >/dev/null 2>&1 || handle_error "Docker not installed"
   ```

2. **Provide helpful usage information**:
   ```bash
   #!/bin/bash
   
   show_usage() {
     echo "Usage: @myhook [options] <argument>"
     echo "Options:"
     echo "  -h, --help     Show this help message"
     echo "  -v, --verbose  Show verbose output"
     exit 0
   }
   
   # Show usage if requested
   if [[ "$1" == "-h" || "$1" == "--help" ]]; then
     show_usage
   fi
   ```

3. **Process arguments properly**:
   ```bash
   #!/bin/bash
   
   # Parse arguments
   VERBOSE=false
   
   while [[ $# -gt 0 ]]; do
     case "$1" in
       -v|--verbose)
         VERBOSE=true
         shift
         ;;
       *)
         # Default case
         ARGUMENT="$1"
         shift
         ;;
     esac
   done
   
   if [[ "$VERBOSE" == true ]]; then
     echo "Running in verbose mode"
   fi
   ```

4. **Provide clear output and return status codes**:
   ```bash
   #!/bin/bash
   
   # Do the main work
   if some_command; then
     echo "Success!"
     exit 0
   else
     echo "Failed!" >&2
     exit 1
   fi
   ```

## Security Considerations

1. **Review scripts before blending**: Always review scripts before adding them to your shell configuration, especially scripts from untrusted sources.

2. **Avoid sensitive information**: Don't include API keys, passwords, or other sensitive information in hook scripts.

3. **Use environment variables** for configuration that might change:
   ```bash
   # Use environment variable with fallback
   export PROJECT_DIR=${PROJECT_DIR:-"$HOME/projects"}
   ```

## Common Use Cases

### Command Aliases (Shell Extension)

Create shortcuts for frequently used commands:

```bash
#!/bin/bash
# Pocket CLI aliases hook

# Basic Pocket commands
alias pk='pocket'
alias pka='pocket add'
alias pkl='pocket list'
alias pks='pocket search'

echo "Pocket CLI aliases loaded!"
```

### Project Setup Script (Executable Hook)

Create a script to set up a new project:

```bash
#!/bin/bash
# Project setup script

# Show usage if no arguments provided
if [ $# -eq 0 ]; then
  echo "Usage: @setup <project_name> [template]"
  echo "Available templates: node, python, rust"
  exit 1
fi

PROJECT_NAME=$1
TEMPLATE=${2:-node}  # Default to node template

# Create project directory
mkdir -p "$PROJECT_NAME"
cd "$PROJECT_NAME" || exit 1

case "$TEMPLATE" in
  node)
    echo "Setting up Node.js project..."
    npm init -y
    npm install express
    echo "Node.js project created successfully!"
    ;;
  python)
    echo "Setting up Python project..."
    python -m venv venv
    echo "Python project created successfully!"
    ;;
  rust)
    echo "Setting up Rust project..."
    cargo init --bin
    echo "Rust project created successfully!"
    ;;
  *)
    echo "Unknown template: $TEMPLATE"
    echo "Available templates: node, python, rust"
    exit 1
    ;;
esac

echo "Project $PROJECT_NAME created with $TEMPLATE template!"
```

## Troubleshooting

### Hook Not Loading

If your shell extension isn't loading when you open a new terminal:

1. Make sure the hook is properly installed:
   ```bash
   pocket blend list
   ```

2. Check if the hook file exists:
   ```bash
   ls -la ~/.pocket/hooks/
   ```

3. Verify that the source line was added to your shell config:
   ```bash
   grep -n "Pocket CLI hook" ~/.zshrc  # or ~/.bashrc
   ```

4. Try sourcing your shell configuration manually:
   ```bash
   source ~/.zshrc  # or ~/.bashrc
   ```

### Executable Hook Not Found

If you can't run your executable hook with the `@` prefix:

1. Make sure the hook is properly installed as executable:
   ```bash
   pocket blend list
   ```
   Look for `[executable]` next to your hook.

2. Check if the wrapper script exists and is executable:
   ```bash
   ls -la ~/.pocket/bin/@*
   ```

3. Verify that the bin directory is in your PATH:
   ```bash
   echo $PATH | grep .pocket/bin
   ```

4. Try running the hook using the blend command:
   ```bash
   pocket blend run hook_name
   ```

## Conclusion

Shell hooks in Pocket CLI provide powerful ways to extend your shell environment and create custom commands. By using both shell extensions and executable hooks, you can significantly enhance your development workflow and productivity.

Remember:
- Use **shell extensions** for aliases, functions, and environment setup
- Use **executable hooks** for scripts you want to run directly with the `@` prefix

Happy blending!