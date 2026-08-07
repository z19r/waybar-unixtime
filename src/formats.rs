use chrono::{DateTime, Local, Utc};

/// Section a format belongs to, mirroring the UnixTime panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Timestamp,
    Date,
}

/// One selectable output format.
pub struct Spec {
    pub key: &'static str,
    pub label: &'static str,
    pub group: Group,
}

/// Every built-in format, in panel order.
pub const SPECS: &[Spec] = &[
    Spec {
        key: "seconds",
        label: "Seconds",
        group: Group::Timestamp,
    },
    Spec {
        key: "millis",
        label: "Milliseconds",
        group: Group::Timestamp,
    },
    Spec {
        key: "micros",
        label: "Microseconds",
        group: Group::Timestamp,
    },
    Spec {
        key: "nanos",
        label: "Nanoseconds",
        group: Group::Timestamp,
    },
    Spec {
        key: "iso-utc",
        label: "ISO 8601 UTC",
        group: Group::Date,
    },
    Spec {
        key: "iso-local",
        label: "ISO 8601 local",
        group: Group::Date,
    },
    Spec {
        key: "iso-date",
        label: "ISO 8601 date",
        group: Group::Date,
    },
    Spec {
        key: "european",
        label: "European",
        group: Group::Date,
    },
    Spec {
        key: "european-short",
        label: "European (short)",
        group: Group::Date,
    },
    Spec {
        key: "us",
        label: "US",
        group: Group::Date,
    },
    Spec {
        key: "us-short",
        label: "US (short)",
        group: Group::Date,
    },
    Spec {
        key: "british",
        label: "British",
        group: Group::Date,
    },
    Spec {
        key: "japanese",
        label: "Japanese",
        group: Group::Date,
    },
    Spec {
        key: "rfc2822",
        label: "RFC 2822",
        group: Group::Date,
    },
    Spec {
        key: "unix-readable",
        label: "Unix readable",
        group: Group::Date,
    },
];

pub fn is_valid_key(key: &str) -> bool {
    key.starts_with("custom:") || SPECS.iter().any(|spec| spec.key == key)
}

/// Render a format key (or `custom:<strftime>`) for the given instant.
pub fn render(
    key: &str,
    utc: DateTime<Utc>,
    local: DateTime<Local>,
) -> Option<String> {
    if let Some(pattern) = key.strip_prefix("custom:") {
        return Some(local.format(pattern).to_string());
    }
    let value = match key {
        "seconds" => utc.timestamp().to_string(),
        "millis" => utc.timestamp_millis().to_string(),
        "micros" => utc.timestamp_micros().to_string(),
        "nanos" => utc
            .timestamp_nanos_opt()
            .map(|n| n.to_string())
            .unwrap_or_else(|| String::from("out of range")),
        "iso-utc" => utc.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "iso-local" => local.format("%Y-%m-%d %H:%M:%S").to_string(),
        "iso-date" => local.format("%Y-%m-%d").to_string(),
        "european" => local.format("%d.%m.%Y %H:%M:%S").to_string(),
        "european-short" => local.format("%d.%m.%Y").to_string(),
        "us" => local.format("%m/%d/%Y %I:%M:%S %p").to_string(),
        "us-short" => local.format("%-m/%-d/%Y").to_string(),
        "british" => local.format("%d/%m/%Y %H:%M:%S").to_string(),
        "japanese" => local.format("%Y年%m月%d日 %H時%M分%S秒").to_string(),
        "rfc2822" => local.to_rfc2822(),
        "unix-readable" => local.format("%a %b %e %H:%M:%S %Z %Y").to_string(),
        _ => return None,
    };
    Some(value)
}

/// Pango tooltip mirroring the UnixTime dropdown: every format with
/// its live value in labelled sections, plus any custom formats.
pub fn tooltip(
    utc: DateTime<Utc>,
    local: DateTime<Local>,
    customs: &[crate::config::Custom],
) -> String {
    let width = SPECS
        .iter()
        .map(|spec| spec.label.chars().count())
        .chain(customs.iter().map(|c| c.name.chars().count()))
        .max()
        .unwrap_or(0);
    let mut out = String::from("<tt>");
    let mut group = None;
    for spec in SPECS {
        if group != Some(spec.group) {
            if group.is_some() {
                out.push('\n');
            }
            let title = match spec.group {
                Group::Timestamp => "TIMESTAMP",
                Group::Date => "DATE FORMATS",
            };
            out.push_str(&format!("<b>── {title} ──</b>\n"));
            group = Some(spec.group);
        }
        let value = render(spec.key, utc, local).unwrap_or_default();
        let pad = " ".repeat(width - spec.label.chars().count() + 2);
        out.push_str(&format!("{}{}{}\n", spec.label, pad, value));
    }
    if !customs.is_empty() {
        out.push_str("\n<b>── CUSTOM ──</b>\n");
        for custom in customs {
            let key = format!("custom:{}", custom.format);
            let value = render(&key, utc, local).unwrap_or_default();
            let pad = " ".repeat(width - custom.name.chars().count() + 2);
            out.push_str(&format!("{}{}{}\n", custom.name, pad, value));
        }
    }
    out.push_str("</tt>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at() -> (DateTime<Utc>, DateTime<Local>) {
        let utc = Utc.timestamp_opt(1_785_407_677, 0).unwrap();
        (utc, utc.with_timezone(&Local))
    }

    #[test]
    fn renders_all_timestamp_precisions() {
        let (utc, local) = at();
        assert_eq!(render("seconds", utc, local).unwrap(), "1785407677");
        let cases = [
            ("millis", "1785407677000"),
            ("micros", "1785407677000000"),
            ("nanos", "1785407677000000000"),
        ];
        for (key, expected) in cases {
            assert_eq!(render(key, utc, local).unwrap(), expected);
        }
    }

    #[test]
    fn renders_iso_utc_with_zulu_suffix() {
        let (utc, local) = at();
        assert_eq!(
            render("iso-utc", utc, local).unwrap(),
            "2026-07-30T10:34:37Z",
        );
    }

    #[test]
    fn renders_every_spec_without_panicking() {
        let (utc, local) = at();
        for spec in SPECS {
            assert!(
                render(spec.key, utc, local).is_some(),
                "format {} failed",
                spec.key,
            );
        }
    }

    #[test]
    fn renders_custom_strftime_patterns() {
        let (utc, local) = at();
        let out = render("custom:%Y!%m", utc, local).unwrap();
        assert_eq!(out, local.format("%Y!%m").to_string());
    }

    #[test]
    fn unknown_key_returns_none() {
        let (utc, local) = at();
        assert!(render("klingon", utc, local).is_none());
    }

    #[test]
    fn validates_keys_including_custom() {
        assert!(is_valid_key("seconds"));
        assert!(is_valid_key("custom:%Y"));
        assert!(!is_valid_key("bogus"));
    }

    #[test]
    fn tooltip_lists_both_sections_and_all_labels() {
        let (utc, local) = at();
        let customs = vec![crate::config::Custom {
            name: String::from("Deploy tag"),
            format: String::from("%Y%m%d-%H%M"),
        }];
        let tip = tooltip(utc, local, &customs);
        assert!(tip.contains("CUSTOM"));
        assert!(tip.contains("Deploy tag"));
        assert!(tip.contains("TIMESTAMP"));
        assert!(tip.contains("DATE FORMATS"));
        for spec in SPECS {
            assert!(tip.contains(spec.label), "missing {}", spec.label);
        }
        assert!(tip.starts_with("<tt>"));
        assert!(tip.ends_with("</tt>"));
    }
}
