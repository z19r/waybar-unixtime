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
    /// Stream waybar JSON lines forever (default command)
    Run(RunArgs),
    /// Print a single waybar JSON line and exit
    Once(RunArgs),
    /// Print the current timestamp (pipe it to wl-copy)
    Copy(RunArgs),
    /// Generate module CSS from the active omarchy theme
    Css(CssArgs),
}

#[derive(Args, Debug, Default, Clone)]
pub struct RunArgs {
    /// Start in milliseconds mode instead of seconds
    #[arg(long)]
    pub millis: bool,

    /// Refresh interval in milliseconds
    #[arg(long, default_value_t = 1000, value_parser = interval_range)]
    pub interval: u64,
}

#[derive(Args, Debug, Default)]
pub struct CssArgs {
    /// Write to ~/.config/waybar/unixtime.css instead of stdout
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
    fn defaults_to_streaming_run_with_one_second_interval() {
        let cli = Cli::parse_from(["waybar-unixtime"]);
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_run_with_millis_and_interval() {
        let cli = Cli::parse_from([
            "waybar-unixtime",
            "run",
            "--millis",
            "--interval",
            "250",
        ]);
        match cli.command {
            Some(Command::Run(args)) => {
                assert!(args.millis);
                assert_eq!(args.interval, 250);
            }
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
