mod cli;
mod config;
mod convert;
mod formats;
mod history;
mod menu;
mod output;
mod picker;
mod state;
mod theme;

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use chrono::{Local, Utc};
use clap::Parser;

use cli::{Cli, Command, InstallArgs, RunArgs};
use output::Line;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Once);
    let result = match command {
        Command::Once => once(),
        Command::Run(args) => run(&args),
        Command::Copy { format } => copy(format.as_deref()),
        Command::Toggle => toggle(),
        Command::Set { format } => set(&format),
        Command::Formats => list_formats(),
        Command::Convert { input } => convert_cmd(&input.join(" ")),
        Command::History { limit } => history_cmd(limit),
        Command::Picker => picker::run(),
        Command::Menu(args) => menu_cmd(&args),
        Command::Css(args) => css(&args),
        Command::Snippet => snippet(),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) if is_broken_pipe(err.as_ref()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("waybar-unixtime: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Downstream closing the pipe (`... | head`) is not an error.
fn is_broken_pipe(err: &(dyn Error + 'static)) -> bool {
    err.downcast_ref::<io::Error>()
        .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
}

fn print_all(content: &str) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(content.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn emit() -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", Line::now().to_json())?;
    stdout.flush()?;
    Ok(())
}

fn once() -> Result<(), Box<dyn Error>> {
    emit()
}

/// Continuous mode for waybar versions that render streaming
/// custom modules correctly (0.15.0 does not — prefer `once`).
fn run(args: &RunArgs) -> Result<(), Box<dyn Error>> {
    loop {
        emit()?;
        thread::sleep(Duration::from_millis(args.interval));
    }
}

fn copy(format: Option<&str>) -> Result<(), Box<dyn Error>> {
    let key = match format {
        Some(key) => key.to_string(),
        None => state::format(),
    };
    let utc = Utc::now();
    let value = formats::render(&key, utc, utc.with_timezone(&Local))
        .ok_or_else(|| format!("unknown format: {key}"))?;
    let entry = history::Entry {
        at: utc.timestamp(),
        format: key,
        value: value.clone(),
    };
    let _ = history::record(&entry, config::load().history_size);
    print_all(&format!("{value}\n"))
}

/// Render every format for a parsed instant — the "Date & Time" tab.
fn convert_cmd(input: &str) -> Result<(), Box<dyn Error>> {
    let instant = picker::parse_input(input.trim())
        .ok_or_else(|| format!("cannot parse: {input}"))?;
    print_all(&format_table(instant))
}

fn history_cmd(limit: usize) -> Result<(), Box<dyn Error>> {
    let entries = history::list(limit);
    if entries.is_empty() {
        return print_all("history is empty\n");
    }
    let mut out = String::new();
    for entry in entries {
        out.push_str(&format!(
            "{}  {}  ({})\n",
            entry.value, entry.format, entry.at,
        ));
    }
    print_all(&out)
}

fn format_table(utc: chrono::DateTime<Utc>) -> String {
    let local = utc.with_timezone(&Local);
    let customs = config::load().customs;
    let width = formats::SPECS
        .iter()
        .map(|spec| spec.key.len())
        .chain(customs.iter().map(|c| c.name.chars().count()))
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for spec in formats::SPECS {
        let value = formats::render(spec.key, utc, local).unwrap_or_default();
        out.push_str(&format!(
            "{:width$}  {}\n",
            spec.key,
            value,
            width = width
        ));
    }
    for custom in customs {
        let key = format!("custom:{}", custom.format);
        let value = formats::render(&key, utc, local).unwrap_or_default();
        out.push_str(&format!(
            "{:width$}  {}\n",
            custom.name,
            value,
            width = width
        ));
    }
    out
}

fn toggle() -> Result<(), Box<dyn Error>> {
    let next = state::toggled(&state::format());
    state::set_format(next)?;
    eprintln!("display format: {next}");
    Ok(())
}

fn set(format: &str) -> Result<(), Box<dyn Error>> {
    if !formats::is_valid_key(format) {
        return Err(format!(
            "unknown format: {format} (see `waybar-unixtime formats`)",
        )
        .into());
    }
    state::set_format(format)?;
    eprintln!("display format: {format}");
    Ok(())
}

fn list_formats() -> Result<(), Box<dyn Error>> {
    let utc = Utc::now();
    let local = utc.with_timezone(&Local);
    let width = formats::SPECS
        .iter()
        .map(|spec| spec.key.len())
        .max()
        .unwrap_or(0);
    let mut out = String::new();
    for spec in formats::SPECS {
        let value = formats::render(spec.key, utc, local).unwrap_or_default();
        out.push_str(&format!(
            "{:width$}  {}\n",
            spec.key,
            value,
            width = width
        ));
    }
    out.push_str(&format!(
        "{:width$}  any strftime pattern\n",
        "custom:<fmt>",
        width = width
    ));
    print_all(&out)
}

fn menu_cmd(args: &InstallArgs) -> Result<(), Box<dyn Error>> {
    let xml = menu::xml(&config::load().customs);
    write_or_print(args, "unixtime-menu.xml", &xml)
}

fn css(args: &InstallArgs) -> Result<(), Box<dyn Error>> {
    let stylesheet = theme::css(&theme::load());
    write_or_print(args, "unixtime.css", &stylesheet)
}

fn snippet() -> Result<(), Box<dyn Error>> {
    let binary = std::env::current_exe()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| String::from("waybar-unixtime"));
    print_all(&menu::snippet(&binary, &config::load().customs))
}

fn write_or_print(
    args: &InstallArgs,
    name: &str,
    content: &str,
) -> Result<(), Box<dyn Error>> {
    if !args.install {
        return print_all(content);
    }
    let target = waybar_dir()?.join(name);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, content)?;
    eprintln!("wrote {}", target.display());
    Ok(())
}

fn waybar_dir() -> Result<PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config/waybar"))
}
