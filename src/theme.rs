use std::env;
use std::fs;
use std::path::PathBuf;

/// Colors pulled from an omarchy theme's `colors.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub accent: String,
    pub foreground: String,
    pub background: String,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            accent: String::from("#c9a554"),
            foreground: String::from("#c2c2b0"),
            background: String::from("#222222"),
        }
    }
}

/// Directory of the active omarchy theme.
///
/// Honors `OMARCHY_THEME_DIR` for overrides and tests, otherwise
/// resolves omarchy's `current/theme` symlink under `~/.config`.
pub fn theme_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("OMARCHY_THEME_DIR") {
        return Some(PathBuf::from(dir));
    }
    let home = env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/omarchy/current/theme"))
}

/// Load the palette from the active omarchy theme, falling back to
/// the built-in default palette when the theme cannot be read.
pub fn load() -> Palette {
    theme_dir()
        .and_then(|dir| fs::read_to_string(dir.join("colors.toml")).ok())
        .and_then(|text| parse(&text))
        .unwrap_or_default()
}


/// Parse an omarchy `colors.toml`, tolerating missing keys.
pub fn parse(text: &str) -> Option<Palette> {
    let value: toml::Value = toml::from_str(text).ok()?;
    let defaults = Palette::default();
    let get = |key: &str, fallback: &str| -> String {
        value            .and_then(|v| v.as_str())
            .map(normalize)
            .unwrap_or_else(|| fallback.to_string())
    };
    Some(Palette {
        accent: get("accent", &defaults.accent),
        foreground: get("foreground", &defaults.foreground),
        background: get("background", &defaults.background),
    })
}

fn normalize(color: &str) -> String {
    let trimmed = color.trim();
    if trimmed.starts_with('#') {
        trimmed.to_string()
    } else {
        format!("#{trimmed}")
    }
}

/// Render the waybar CSS snippet for a palette.
///
/// The snippet only uses `unixtime-*` named colors, so it can be
/// `@import`-ed into any waybar stylesheet without collisions. Every
/// omarchy theme switch just regenerates this file.
pub fn css(palette: &Palette) -> String {
    format!(
        "/* waybar-unixtime — generated from the active omarchy theme.\n\
        \x20* Regenerate with: waybar-unixtime css --install\n\
        \x20*/\n\
        @define-color unixtime-accent {accent};\n\
        @define-color unixtime-fg {fg};\n\
        @define-color unixtime-bg {bg};\n\
        \n\
        #custom-unixtime {{\n\
        \x20 color: @unixtime-fg;\n\
        \x20 padding: 0 12px;\n\
        \x20 font-family: monospace;\n\
        }}\n\
        \n\
        #custom-unixtime.millis,\n\
        #custom-unixtime.micros,\n\
        #custom-unixtime.nanos,\n\
        #custom-unixtime.custom {{\n\
        \x20 color: @unixtime-accent;\n\
        }}\n\
        \n\
        tooltip {{\n\
        \x20 background: @unixtime-bg;\n\
        \x20 color: @unixtime-fg;\n\
        \x20 border: 1px solid alpha(@unixtime-accent, 0.5);\n\
        \x20 border-radius: 12px;\n\
        }}\n\
        \n\
        menu {{\n\
        \x20 background: @unixtime-bg;\n\
        \x20 color: @unixtime-fg;\n\
        \x20 border: 1px solid alpha(@unixtime-accent, 0.5);\n\
        \x20 border-radius: 12px;\n\
        \x20 padding: 8px 4px;\n\
        }}\n\
        \n\
        menu menuitem {{\n\
        \x20 padding: 5px 12px;\n\
        \x20 border-radius: 8px;\n\
        \x20 font-family: monospace;\n\
        }}\n\
        \n\
        menu menuitem:hover {{\n\
        \x20 background: alpha(@unixtime-accent, 0.25);\n\
        }}\n\
        \n\
        menu separator {{\n\
        \x20 background: alpha(@unixtime-fg, 0.15);\n\
        \x20 margin: 6px 10px;\n\
        }}\n",
        accent = palette.accent,
        fg = palette.foreground,
        bg = palette.background,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r##"
accent = "#78824b"
foreground = "c2c2b0"
background = "#222222"
color0 = "#000000"
"##;

    #[test]
    fn parses_omarchy_colors_and_normalizes_missing_hash() {
        let palette = parse(SAMPLE).unwrap();
        assert_eq!(palette.accent, "#78824b");
        assert_eq!(palette.foreground, "#c2c2b0");
        assert_eq!(palette.background, "#222222");
    }

    #[test]
    fn missing_keys_fall_back_to_defaults() {
        let palette = parse("accent = \"#111111\"").unwrap();
        assert_eq!(palette.accent, "#111111");
        assert_eq!(palette.foreground, Palette::default().foreground);
    }

    #[test]
    fn invalid_toml_yields_none() {
        assert!(parse("not [ valid").is_none());
    }

    #[test]
    fn css_uses_namespaced_define_colors() {
        let out = css(&Palette::default());
        assert!(out.contains("@define-color unixtime-accent #c9a554;"));
        assert!(out.contains("#custom-unixtime {"));
        assert!(out.contains("#custom-unixtime.millis,"));
    }

    #[test]
    fn env_override_wins_for_theme_dir() {
        env::set_var("OMARCHY_THEME_DIR", "/tmp/theme-under-test");
        let dir = theme_dir().unwrap();
        env::remove_var("OMARCHY_THEME_DIR");
        assert_eq!(dir, PathBuf::from("/tmp/theme-under-test"));
    }
}
