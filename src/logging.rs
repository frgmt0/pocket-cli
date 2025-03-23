use colored::{ColoredString, Colorize};
use log::{Level, LevelFilter};
use std::io::Write;
use chrono::Local;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init(level: LevelFilter) {
    INIT.call_once(|| {
        env_logger::Builder::new()
            .format(|buf, record| {
                let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                
                let level_str = match record.level() {
                    Level::Error => "ERROR".red().bold(),
                    Level::Warn => "WARN ".yellow().bold(),
                    Level::Info => "INFO ".green(),
                    Level::Debug => "DEBUG".blue(),
                    Level::Trace => "TRACE".magenta(),
                };
                
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

pub fn info(msg: &str) {
    println!("{} {}", "INFO".green(), msg);
}

pub fn success(msg: &str) {
    println!("{} {}", "SUCCESS".green().bold(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", "WARNING".yellow().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "ERROR".red().bold(), msg);
}

pub fn _cmd_text(cmd: &str, args: &[&str]) -> ColoredString {
    format!("{} {}", cmd, args.join(" ")).cyan()
}

pub fn header(text: &str) -> ColoredString {
    text.blue().bold()
}

pub fn key(text: &str) -> ColoredString {
    text.yellow()
}

pub fn _value(text: &str) -> ColoredString {
    text.white()
}

pub fn _path(text: &str) -> ColoredString {
    text.underline().white()
}

pub fn _id(text: &str) -> ColoredString {
    text.green()
}

pub fn title(text: &str) -> ColoredString {
    text.cyan().bold()
} 