use std::error::Error;
use std::io::Write;
use std::process::{Command, Stdio};

use chrono::{DateTime, Local, Utc};

use crate::{config, convert, formats, history};

const CONVERT_ENTRY: &str = "⇄  Convert / Date & Time…";
const HISTORY_ENTRY: &str = "⟲  History…";

/// Launcher candidates, tried in order. Omarchy ships walker.
const LAUNCHERS: &[&[&str]] = &[
    &["walker", "--dmenu", "-p"],
    &["fuzzel", "--dmenu", "-p"],
    &["wofi", "--dmenu", "-p"],
    &["rofi", "-dmenu", "-p"],
];

/// Interactive panel: formats, converter, and history in a dmenu.
pub fn run() -> Result<(), Box<dyn Error>> {
    let mut lines = vec![CONVERT_ENTRY.to_string(), HISTORY_ENTRY.to_string()];
    let utc = Utc::now();
    lines.extend(format_lines(utc));
    let Some(choice) = dmenu("unixtime", &lines)? else {
        return Ok(());
    };
    match choice.as_str() {
        CONVERT_ENTRY => convert_flow(),
        HISTORY_ENTRY => history_flow(),
        other => copy_value(value_of(other)),
    }
}

/// "Date & Time" tab: free-text input -> every format -> copy.
fn convert_flow() -> Result<(), Box<dyn Error>> {
    let hint = vec![
        String::from("now"),
        String::from("now +2h30m"),
        String::from("1786098721"),
        String::from("2026-08-07 10:00"),
        String::from("30.07.2026 12:34:56"),
    ];
    let Some(raw) = dmenu("epoch or date", &hint)? else {
        return Ok(());
    };
    let instant = parse_input(raw.trim())
        .ok_or_else(|| format!("cannot parse: {raw}"))?;
    let lines = format_lines(instant);
    let Some(choice) = dmenu("copy", &lines)? else {
        return Ok(());
    };
    copy_value(value_of(&choice))
}

/// "History" tab: newest copies first, select to re-copy.
fn history_flow() -> Result<(), Box<dyn Error>> {
    let entries = history::list(50);
    if entries.is_empty() {
        let _ = dmenu("history", &[String::from("(empty)")])?;
        return Ok(());
    }
    let lines: Vec<String> = entries
        .iter()
        .map(|entry| {
            format!("{}  ·  {} {}", entry.value, entry.format, ago(entry))
        })
        .collect();
    let Some(choice) = dmenu("history", &lines)? else {
        return Ok(());
    };
    let value = choice.split("  ·  ").next().unwrap_or(&choice);
    copy_value(value.to_string())
}

/// Full-string dates win ("2026-08-07 10:00"); otherwise the last
/// token may be an offset ("now +2h30m", "1786098721 -1d").
pub fn parse_input(raw: &str) -> Option<DateTime<Utc>> {
    if let Some(instant) = convert::parse(raw) {
        return Some(instant);
    }
    let (head, tail) = raw.rsplit_once(char::is_whitespace)?;
    let base = convert::parse(head.trim())?;
    convert::offset(base, tail.trim())
}

fn ago(entry: &history::Entry) -> String {
    let delta = Utc::now().timestamp() - entry.at;
    match delta {
        d if d < 90 => format!("({d}s ago)"),
        d if d < 5_400 => format!("({}m ago)", d / 60),
        d if d < 129_600 => format!("({}h ago)", d / 3_600),
        d => format!("({}d ago)", d / 86_400),
    }
}

/// One aligned "Label  value" line per format, customs included.
fn format_lines(utc: DateTime<Utc>) -> Vec<String> {
    let local = utc.with_timezone(&Local);
    let customs = config::load().customs;
    let width = formats::SPECS
        .iter()
        .map(|spec| spec.label.chars().count())
        .chain(customs.iter().map(|c| c.name.chars().count()))
        .max()
        .unwrap_or(0);
    let mut lines: Vec<String> = formats::SPECS
        .iter()
        .map(|spec| {
            let value =
                formats::render(spec.key, utc, local).unwrap_or_default();
            aligned(spec.label, &value, width)
        })
        .collect();
    for custom in customs {
        let key = format!("custom:{}", custom.format);
        let value = formats::render(&key, utc, local).unwrap_or_default();
        lines.push(aligned(&custom.name, &value, width));
    }
    lines
}

fn aligned(label: &str, value: &str, width: usize) -> String {
    let pad = " ".repeat(width - label.chars().count() + 2);
    format!("{label}{pad}{value}")
}

fn value_of(line: &str) -> String {
    line.rsplit("  ").next().unwrap_or(line).trim().to_string()
}

fn copy_value(value: String) -> Result<(), Box<dyn Error>> {
    if value.is_empty() {
        return Ok(());
    }
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("wl-copy: {e}"))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(value.as_bytes())?;
    }
    child.wait()?;
    let entry = history::Entry {
        at: Utc::now().timestamp(),
        format: String::from("picker"),
        value,
    };
    let _ = history::record(&entry, config::load().history_size);
    Ok(())
}

/// Run the first available dmenu-style launcher.
fn dmenu(
    prompt: &str,
    lines: &[String],
) -> Result<Option<String>, Box<dyn Error>> {
    for candidate in LAUNCHERS {
        let (bin, args) = (candidate[0], &candidate[1..]);
        let spawned = Command::new(bin)
            .args(args)
            .arg(prompt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();
        let Ok(mut child) = spawned else {
            continue;
        };
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(lines.join("\n").as_bytes());
        }
        let output = child.wait_with_output()?;
        let choice = String::from_utf8_lossy(&output.stdout)
            .trim_end_matches('\n')
            .to_string();
        return Ok(if choice.is_empty() {
            None
        } else {
            Some(choice)
        });
    }
    Err("no dmenu launcher found (walker/fuzzel/wofi/rofi)".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_of_takes_the_last_column() {
        assert_eq!(value_of("Seconds     1786098721"), "1786098721");
        assert_eq!(
            value_of("ISO 8601 UTC  2026-08-07T10:30:06Z"),
            "2026-08-07T10:30:06Z",
        );
    }

    #[test]
    fn format_lines_cover_all_specs() {
        let lines = format_lines(Utc::now());
        assert!(lines.len() >= formats::SPECS.len());
    }
}
