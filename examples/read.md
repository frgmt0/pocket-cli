# Pocket CLI Examples

This directory contains example scripts and workflows to demonstrate the capabilities of Pocket CLI's script execution feature.

## Python Project Setup

The `python-project.pocket` workflow demonstrates how to set up a complete Python project environment using:

1. Script execution (`execute` command)
2. Template insertion (`insert` command)

### How to Use

I can run the workflow with:

```bash
pocket lint examples/python-project.pocket
```

This will:
1. Execute the Python project setup script, which:
   - Creates a new Python project with modern best practices
   - Sets up a virtual environment using `uv`
   - Creates all necessary files and directories
   - Prompts for project name, location, and Python version for me

2. Insert a template from your templates backpack into the project's `spec.md` file
   - You'll be prompted to enter the path to the `spec.md` file
   - The script provides this path in its output

### Test It Yourself

Try running the test script without making it executable first:

```bash
pocket execute -f examples/test.sh
```

You'll see that it executes successfully, even though it doesn't have executable permissions.

## Customizing for Your Needs

You can modify these examples to create your own workflows:

1. Edit the Python project setup script to use different tools or configurations
2. Create additional workflow files for different types of projects
3. Combine multiple pocket commands in a single workflow

## Using Scripts Outside of Workflows

You can also run the scripts directly:

```bash
pocket execute -f examples/simple-python-setup.sh
```

And insert templates manually:

```bash
pocket insert <template-id> --file path/to/file
``` 