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
    /// Badge chip text, like the UnixTime app (TS / ISO / EU / …).
    pub badge: &'static str,
    /// Badge chip color.
    pub color: &'static str,
}

pub const BADGE_TS: &str = "#3b82f6";
pub const BADGE_ISO: &str = "#c026d3";
pub const BADGE_EU: &str = "#8b5cf6";
pub const BADGE_US: &str = "#ef4444";
pub const BADGE_UK: &str = "#f97316";
pub const BADGE_JP: &str = "#14b8a6";
pub const BADGE_MISC: &str = "#64748b";
pub const BADGE_CUSTOM: &str = "#d946ef";

macro_rules! spec {
    ($key:expr, $label:expr, $group:expr, $badge:expr, $color:expr) => {
        Spec {
            key: $key,
            label: $label,
            group: $group,
            badge: $badge,
            color: $color,
        }
    };
}

/// Every built-in format, in panel order.
pub const SPECS: &[Spec] = &[
    spec!("seconds", "Seconds", Group::Timestamp, "TS", BADGE_TS),
    spec!("millis", "Milliseconds", Group::Timestamp, "TS", BADGE_TS),
    spec!("micros", "Microseconds", Group::Timestamp, "TS", BADGE_TS),
    spec!("nanos", "Nanoseconds", Group::Timestamp, "TS", BADGE_TS),
    spec!("iso-utc", "ISO 8601 UTC", Group::Date, "ISO", BADGE_ISO),
    spec!("iso-local", "ISO 8601 local", Group::Date, "ISO", BADGE_ISO),
    spec!("iso-date", "ISO 8601 date", Group::Date, "ISO", BADGE_ISO),
    spec!("european", "European", Group::Date, "EU", BADGE_EU),
    spec!(
        "european-short",
        "European (short)",
        Group::Date,
        "EU",
        BADGE_EU
    ),
    spec!("us", "US", Group::Date, "US", BADGE_US),
    spec!("us-short", "US (short)", Group::Date, "US", BADGE_US),
    spec!("british", "British", Group::Date, "UK", BADGE_UK),
    spec!("japanese", "Japanese", Group::Date, "JP", BADGE_JP),
    spec!("rfc2822", "RFC 2822", Group::Date, "RFC", BADGE_MISC),
    spec!(
        "unix-readable",
        "Unix readable",
        Group::Date,
        "UNIX",
        BADGE_MISC
    ),
];

/// A Pango badge chip: colored pill with white text, fixed width
/// so <tt> columns stay aligned.
pub fn badge_markup(badge: &str, color: &str) -> String {
    format!(
        "<span background=\"{color}\" foreground=\"#ffffff\">\
         <b> {badge:^4} </b></span>",
    )
}

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

fn section_header(title: &str) -> String {
    format!(
        "<span foreground=\"#8a8f98\" size=\"small\">\
         ─────  <b>{title}</b>  ─────</span>\n",
    )
}

fn panel_line(
    active: bool,
    badge: &str,
    color: &str,
    label: &str,
    value: &str,
    width: usize,
) -> String {
    let check = if active { "✓" } else { " " };
    let pad = " ".repeat(width - label.chars().count() + 2);
    format!(
        "{check} {chip} {label}{pad}{value}\n",
        chip = badge_markup(badge, color),
    )
}

/// Pango tooltip mirroring the UnixTime dropdown: badge chips,
/// a ✓ on the active format, live values, and custom formats.
pub fn tooltip(
    utc: DateTime<Utc>,
    local: DateTime<Local>,
    customs: &[crate::config::Custom],
    active: &str,
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
            out.push_str(&section_header(title));
            group = Some(spec.group);
        }
        let value = render(spec.key, utc, local).unwrap_or_default();
        out.push_str(&panel_line(
            spec.key == active,
            spec.badge,
            spec.color,
            spec.label,
            &value,
            width,
        ));
    }
    if !customs.is_empty() {
        out.push('\n');
        out.push_str(&section_header("CUSTOM"));
        for custom in customs {
            let key = format!("custom:{}", custom.format);
            let value = render(&key, utc, local).unwrap_or_default();
            out.push_str(&panel_line(
                key == active,
                "FMT",
                BADGE_CUSTOM,
                &custom.name,
                &value,
                width,
            ));
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
        let tip = tooltip(utc, local, &customs, "millis");
        assert!(tip.contains("CUSTOM"));
        assert!(tip.contains("Deploy tag"));
        assert!(tip.contains("✓"));
        assert!(tip.contains(&badge_markup("TS", BADGE_TS)));
        let check_line = tip
            .lines()
            .find(|line| line.starts_with('✓'))
            .expect("active line marked");
        assert!(check_line.contains("Milliseconds"));
        assert!(tip.contains("TIMESTAMP"));
        assert!(tip.contains("DATE FORMATS"));
        for spec in SPECS {
            assert!(tip.contains(spec.label), "missing {}", spec.label);
        }
        assert!(tip.starts_with("<tt>"));
        assert!(tip.ends_with("</tt>"));
    }
}
