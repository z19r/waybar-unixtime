use std::env;
use std::fs;
use std::path::PathBuf;

use crate::formats;

pub const DEFAULT_FORMAT: &str = "seconds";

/// Serializes tests that mutate the process-global state-dir env
/// var; cargo runs tests in parallel threads.
#[cfg(test)]
pub static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Where the selected display format is persisted.
///
/// Honors `WAYBAR_UNIXTIME_STATE_DIR` (tests), then `XDG_STATE_HOME`,
/// then `~/.local/state`.
fn state_file() -> Option<PathBuf> {
    if let Ok(dir) = env::var("WAYBAR_UNIXTIME_STATE_DIR") {
        return Some(PathBuf::from(dir).join("format"));
    }
    let base = match env::var_os("XDG_STATE_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env::var_os("HOME")?).join(".local/state"),
    };
    Some(base.join("waybar-unixtime/format"))
}

/// Current display format, falling back to the configured default
/// (Settings tab), then to seconds.
pub fn format() -> String {
    state_file()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|raw| raw.trim().to_string())
        .filter(|key| formats::is_valid_key(key))
        .unwrap_or_else(|| {
            let configured = crate::config::load().default_format;
            if formats::is_valid_key(&configured) {
                configured
            } else {
                DEFAULT_FORMAT.to_string()
            }
        })
}

/// Persist a new display format.
pub fn set_format(key: &str) -> std::io::Result<()> {
    let Some(path) = state_file() else {
        return Err(std::io::Error::other("cannot resolve state dir"));
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, key)
}

/// Flip between seconds and milliseconds; any other format returns
/// to seconds.
pub fn toggled(current: &str) -> &'static str {
    if current == "seconds" {
        "millis"
    } else {
        "seconds"
    }
}

/// CSS class for the current format (custom patterns share one).
pub fn class(key: &str) -> String {
    if key.starts_with("custom:") {
        String::from("custom")
    } else {
        key.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_flips_seconds_and_millis() {
        assert_eq!(toggled("seconds"), "millis");
        assert_eq!(toggled("millis"), "seconds");
        assert_eq!(toggled("iso-utc"), "seconds");
    }

    #[test]
    fn class_collapses_custom_patterns() {
        assert_eq!(class("seconds"), "seconds");
        assert_eq!(class("custom:%Y"), "custom");
    }

    // One test mutates the process-global env var serially: parallel
    // tests sharing WAYBAR_UNIXTIME_STATE_DIR would race each other.
    #[test]
    fn state_dir_roundtrip_and_invalid_fallback() {
        let _guard = TEST_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join("wut-state-test");
        env::set_var("WAYBAR_UNIXTIME_STATE_DIR", &dir);

        set_format("iso-utc").unwrap();
        assert_eq!(format(), "iso-utc");

        std::fs::write(dir.join("format"), "nonsense").unwrap();
        assert_eq!(format(), DEFAULT_FORMAT);

        env::remove_var("WAYBAR_UNIXTIME_STATE_DIR");
    }
}
