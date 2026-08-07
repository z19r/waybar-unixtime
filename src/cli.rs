use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "waybar-unixtime",
    version,
    about = "Live unix timestamps for Waybar, themed by omarchy",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Print a single waybar JSON line and exit (use with interval)
    Once,
    /// Stream waybar JSON lines forever (continuous exec mode)
    Run(RunArgs),
    /// Print a timestamp in FORMAT (default: current display format)
    Copy {
        /// Format key (see `formats`) or custom:<strftime>
        format: Option<String>,
    },
    /// Flip the display format between seconds and milliseconds
    Toggle,
    /// Set the display format shown in the bar
    Set {
        /// Format key (see `formats`) or custom:<strftime>
        format: String,
    },
    /// List all format keys with live example output
    Formats,
    /// Generate the click dropdown menu XML
    Menu(InstallArgs),
    /// Generate module CSS from the active omarchy theme
    Css(InstallArgs),
    /// Print a ready-to-paste waybar module config
    Snippet,
}

#[derive(Args, Debug, Default, Clone)]
pub struct RunArgs {
    /// Refresh interval in milliseconds
    #[arg(long, default_value_t = 1000, value_parser = interval_range)]
    pub interval: u64,
}

#[derive(Args, Debug, Default)]
pub struct InstallArgs {
    /// Write into ~/.config/waybar/ instead of stdout
    #[arg(long)]
    pub install: bool,
}

fn interval_range(raw: &str) -> Result<u64, String> {
    let value: u64 = raw.parse().map_err(|_| "not a number".to_string())?;
    if (50..=3_600_000).contains(&value) {
        Ok(value)
    } else {
        Err(String::from("interval must be 50..=3600000 ms"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_no_subcommand() {
        let cli = Cli::parse_from(["waybar-unixtime"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_copy_with_format_argument() {
        let cli = Cli::parse_from(["waybar-unixtime", "copy", "iso-utc"]);
        match cli.command {
            Some(Command::Copy { format }) => {
                assert_eq!(format.as_deref(), Some("iso-utc"));
            }
            other => panic!("expected copy, got {other:?}"),
        }
    }

    #[test]
    fn parses_run_with_interval() {
        let cli =
            Cli::parse_from(["waybar-unixtime", "run", "--interval", "250"]);
        match cli.command {
            Some(Command::Run(args)) => assert_eq!(args.interval, 250),
            other => panic!("expected run, got {other:?}"),
        }
    }

    #[test]
    fn rejects_out_of_range_interval() {
        let result =
            Cli::try_parse_from(["waybar-unixtime", "run", "--interval", "10"]);
        assert!(result.is_err());
    }
}
