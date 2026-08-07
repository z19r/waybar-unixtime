use chrono::{Local, Utc};
use serde::Serialize;

use crate::{config, formats, state};

/// One line of waybar custom-module JSON output.
#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub tooltip: String,
    pub class: String,
}

impl Line {
    /// Render "now" using the persisted display format.
    pub fn now() -> Line {
        Line::at_format(&state::format())
    }

    pub fn at_format(key: &str) -> Line {
        let utc = Utc::now();
        let local = utc.with_timezone(&Local);
        Line {
            text: formats::render(key, utc, local)
                .unwrap_or_else(|| String::from("?")),
            tooltip: formats::tooltip(utc, local, &config::load().customs),
            class: state::class(key),
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Serialization of plain strings cannot fail; keep a
            // harmless fallback rather than panicking inside waybar.
            String::from("{\"text\":\"?\"}")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_all_waybar_fields() {
        let line = Line {
            text: String::from("1700000000"),
            tooltip: String::from("tip"),
            class: String::from("seconds"),
        };
        assert_eq!(
            line.to_json(),
            "{\"text\":\"1700000000\",\"tooltip\":\"tip\",\
             \"class\":\"seconds\"}",
        );
    }

    #[test]
    fn at_format_millis_is_numeric_with_matching_class() {
        let line = Line::at_format("millis");
        assert!(line.text.chars().all(|c| c.is_ascii_digit()));
        assert!(line.text.len() >= 13);
        assert_eq!(line.class, "millis");
        assert!(line.tooltip.contains("DATE FORMATS"));
    }
}
