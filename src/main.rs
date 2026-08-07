mod cli;
mod clock;
mod output;
mod theme;

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use chrono::Utc;
use clap::Parser;
use signal_hook::consts::signal::SIGUSR1;

use cli::{Cli, Command, CssArgs, RunArgs};
use clock::Format;
use output::Line;

/// Poll granularity for signal handling inside the tick loop.
const POLL_MS: u64 = 100;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Run(RunArgs::default()));
    let result = match command {
        Command::Run(args) => run(&args),
        Command::Once(args) => once(&args),
        Command::Copy(args) => copy(&args),
        Command::Css(args) => css(&args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("waybar-unixtime: {err}");
            ExitCode::FAILURE
        }
    }
}

fn start_format(args: &RunArgs) -> Format {
    if args.millis {
        Format::Millis
    } else {
        Format::Seconds
    }
}

fn emit(format: Format) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{}", Line::now(format).to_json())?;
    stdout.flush()?;
    Ok(())
}

/// Stream JSON lines forever; SIGUSR1 toggles seconds <-> millis.
fn run(args: &RunArgs) -> Result<(), Box<dyn Error>> {
    let toggle = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGUSR1, Arc::clone(&toggle))?;

    let mut format = start_format(args);
    loop {
        emit(format)?;
        let mut slept = 0;
        while slept < args.interval {
            thread::sleep(Duration::from_millis(
                POLL_MS.min(args.interval - slept),
            ));
            slept += POLL_MS;
            if toggle.swap(false, Ordering::Relaxed) {
                format = format.toggled();
                break;
            }
        }
    }
}

fn once(args: &RunArgs) -> Result<(), Box<dyn Error>> {
    emit(start_format(args))
}

/// Print the raw timestamp for piping into a clipboard tool.
fn copy(args: &RunArgs) -> Result<(), Box<dyn Error>> {
    println!("{}", clock::text(Utc::now(), start_format(args)));
    Ok(())
}

fn css(args: &CssArgs) -> Result<(), Box<dyn Error>> {
    let stylesheet = theme::css(&theme::load());
    if !args.install {
        print!("{stylesheet}");
        return Ok(());
    }
    let target = css_target()?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&target, stylesheet)?;
    eprintln!("wrote {}", target.display());
    eprintln!("add to waybar style.css:  @import \"unixtime.css\";");
    Ok(())
}

fn css_target() -> Result<PathBuf, Box<dyn Error>> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config/waybar/unixtime.css"))
}
