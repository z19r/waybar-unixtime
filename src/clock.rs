use chrono::{DateTime, Local, SecondsFormat, Utc};

/// Display format for the bar text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Format {
    #[default]
    Seconds,
    Millis,
}

impl Format {
    pub fn toggled(self) -> Self {
        match self {
            Format::Seconds => Format::Millis,
            Format::Millis => Format::Seconds,
        }
    }

    /// CSS class waybar attaches to the module for this format.
    pub fn class(self) -> &'static str {
        match self {
            Format::Seconds => "seconds",
            Format::Millis => "millis",
        }
    }
}

pub fn text(now: DateTime<Utc>, format: Format) -> String {
    match format {
        Format::Seconds => now.timestamp().to_string(),
        Format::Millis => now.timestamp_millis().to_string(),
    }
}

pub fn tooltip(utc: DateTime<Utc>, local: DateTime<Local>) -> String {
    let iso = utc.to_rfc3339_opts(SecondsFormat::Secs, true);
    format!(
        "<b>{}</b> seconds since the epoch\n\
         UTC    {}\n\
         Local  {}\n\
         ISO    {}",
        utc.timestamp(),
        utc.format("%Y-%m-%d %H:%M:%S"),
        local.format("%Y-%m-%d %H:%M:%S %Z"),
        iso,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 500_000_000).unwrap()
    }

    #[test]
    fn formats_seconds_as_plain_integer() {
        assert_eq!(text(at(1_700_000_000), Format::Seconds), "1700000000");
    }

    #[test]
    fn formats_millis_including_subsecond_part() {
        assert_eq!(text(at(1_700_000_000), Format::Millis), "1700000000500");
    }

    #[test]
    fn toggling_flips_between_both_formats() {
        assert_eq!(Format::Seconds.toggled(), Format::Millis);
        assert_eq!(Format::Millis.toggled(), Format::Seconds);
    }

    #[test]
    fn classes_match_format_names() {
        assert_eq!(Format::Seconds.class(), "seconds");
        assert_eq!(Format::Millis.class(), "millis");
    }

    #[test]
    fn tooltip_contains_epoch_utc_and_iso_lines() {
        let utc = at(1_700_000_000);
        let tip = tooltip(utc, utc.with_timezone(&Local));
        assert!(tip.contains("<b>1700000000</b>"));
        assert!(tip.contains("UTC    2023-11-14 22:13:20"));
        assert!(tip.contains("ISO    2023-11-14T22:13:20Z"));
    }
}
