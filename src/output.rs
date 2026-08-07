use chrono::{Local, Utc};
use serde::Serialize;

use crate::clock::{self, Format};

/// One line of waybar custom-module JSON output.
#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub tooltip: String,
    pub class: &'static str,
}

impl Line {
    pub fn now(format: Format) -> Line {
        let utc = Utc::now();
        Line {
            text: clock::text(utc, format),
            tooltip: clock::tooltip(utc, utc.with_timezone(&Local)),
            class: format.class(),
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
            class: "seconds",
        };
        assert_eq!(
            line.to_json(),
            "{\"text\":\"1700000000\",\"tooltip\":\"tip\",\
             \"class\":\"seconds\"}",
        );
    }

    #[test]
    fn now_produces_numeric_text_and_matching_class() {
        let line = Line::now(Format::Millis);
        assert!(line.text.chars().all(|c| c.is_ascii_digit()));
        assert!(line.text.len() >= 13);
        assert_eq!(line.class, "millis");
    }
}
