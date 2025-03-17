use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter};
use std::io::Write;
use chrono::Local;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the logger with the given log level
pub fn init(level: LevelFilter) {
    INIT.call_once(|| {
        env_logger::Builder::new()
            .format(|buf, record| {
                // Format the timestamp
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                
                // Format the level
                let level_str = match record.level() {
                    Level::Error => "ERROR".red().bold(),
                    Level::Warn => "WARN ".yellow().bold(),
                    Level::Info => "INFO ".green(),
                    Level::Debug => "DEBUG".blue(),
                    Level::Trace => "TRACE".magenta(),
                };
                
                // Format the module path
                let target = if let Some(target) = record.module_path() {
                    if target.contains("::") {
                        let parts: Vec<&str> = target.split("::").collect();
                        let prefix = parts[0];
                        let suffix = parts.last().unwrap_or(&"");
                        format!("{}.{}", prefix, suffix)
                    } else {
                        target.to_string()
                    }
                } else {
                    "pocket".to_string()
                };
                
                // Format the message
                writeln!(
                    buf,
                    "{} {} {} > {}",
                    timestamp.dimmed(),
                    level_str,
                    target.dimmed(),
                    record.args()
                )
            })
            .filter(None, level)
            .init();
        
        log::info!("Logger initialized at level {}", level);
    });
}

/// Log an info message directly to terminal (not through the logger)
pub fn info(msg: &str) {
    println!("{} {}", "INFO".green(), msg);
}

/// Log a success message directly to terminal (not through the logger)
pub fn success(msg: &str) {
    println!("{} {}", "SUCCESS".green().bold(), msg);
}

/// Log a warning message directly to terminal (not through the logger)
pub fn warning(msg: &str) {
    println!("{} {}", "WARNING".yellow().bold(), msg);
}

/// Log an error message directly to terminal (not through the logger)
pub fn error(msg: &str) {
    eprintln!("{} {}", "ERROR".red().bold(), msg);
}

/// Format text for command output
pub fn cmd_text(cmd: &str, args: &[&str]) -> ColoredString {
    format!("{} {}", cmd, args.join(" ")).cyan()
}

/// Format a header for sections
pub fn header(text: &str) -> ColoredString {
    text.blue().bold()
}

/// Format a key for key-value output
pub fn key(text: &str) -> ColoredString {
    text.yellow()
}

/// Format a value for key-value output
pub fn value(text: &str) -> ColoredString {
    text.white()
}

/// Format a path for display
pub fn path(text: &str) -> ColoredString {
    text.underline().white()
}

/// Format an ID for display
pub fn id(text: &str) -> ColoredString {
    text.green()
}

/// Format a title for display
pub fn title(text: &str) -> ColoredString {
    text.cyan().bold()
} 