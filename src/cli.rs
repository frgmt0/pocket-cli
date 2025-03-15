mod commands {
    pub mod blend;
}

pub fn build_cli() -> Command {
    let blend_command = Command::new("blend")
        .about("Blend shell scripts into your shell configuration")
        .arg_required_else_help(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("script_file")
                .help("Path to shell script file to blend into shell configuration")
                .required(false)
        )
        .subcommand(
            Command::new("edit")
                .about("Edit an existing hook")
                .arg(
                    Arg::new("hook_name")
                        .help("Name of the hook to edit (with or without @ prefix)")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("list")
                .about("List all installed hooks")
        );

    // ... existing code ...

    blend_command
} 