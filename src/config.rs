use std::env;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

/// User settings — the "Settings" and "Custom Formats" tabs.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    /// Format shown in the bar when no state has been saved yet.
    pub default_format: String,
    /// Max entries kept in the copy history (0 disables history).
    pub history_size: usize,
    /// Named custom strftime formats, shown alongside built-ins.
    #[serde(rename = "custom")]
    pub customs: Vec<Custom>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Custom {
    pub name: String,
    pub format: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_format: String::from("seconds"),
            history_size: 50,
            customs: Vec::new(),
        }
    }
}

/// Honors `WAYBAR_UNIXTIME_CONFIG_DIR` (tests), then
/// `XDG_CONFIG_HOME`, then `~/.config`.
fn config_file() -> Option<PathBuf> {
    if let Ok(dir) = env::var("WAYBAR_UNIXTIME_CONFIG_DIR") {
        return Some(PathBuf::from(dir).join("config.toml"));
    }
    let base = match env::var_os("XDG_CONFIG_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env::var_os("HOME")?).join(".config"),
    };
    Some(base.join("waybar-unixtime/config.toml"))
}

/// Load settings, falling back to defaults on any problem.
pub fn load() -> Config {
    config_file()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_seconds_and_fifty_entries() {
        let config = Config::default();
        assert_eq!(config.default_format, "seconds");
        assert_eq!(config.history_size, 50);
        assert!(config.customs.is_empty());
    }

    #[test]
    fn parses_partial_config_with_customs() {
        let config: Config = toml::from_str(
            "default_format = \"iso-utc\"\n\
             [[custom]]\n\
             name = \"Deploy tag\"\n\
             format = \"%Y%m%d-%H%M\"\n",
        )
        .unwrap();
        assert_eq!(config.default_format, "iso-utc");
        assert_eq!(config.history_size, 50);
        assert_eq!(config.customs[0].name, "Deploy tag");
        assert_eq!(config.customs[0].format, "%Y%m%d-%H%M");
    }

    #[test]
    fn garbage_config_falls_back_to_defaults() {
        let dir = std::env::temp_dir().join("wut-config-garbage");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.toml"), "not [ toml").unwrap();
        env::set_var("WAYBAR_UNIXTIME_CONFIG_DIR", &dir);
        let config = load();
        env::remove_var("WAYBAR_UNIXTIME_CONFIG_DIR");
        assert_eq!(config, Config::default());
    }
}
