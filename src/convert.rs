use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};

/// Date-string patterns tried in order — the "Date & Time" tab.
const DATETIME_PATTERNS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%d.%m.%Y %H:%M:%S",
    "%d.%m.%Y %H:%M",
    "%m/%d/%Y %I:%M:%S %p",
    "%m/%d/%Y %H:%M:%S",
    "%m/%d/%Y %H:%M",
];

const DATE_PATTERNS: &[&str] = &["%Y-%m-%d", "%d.%m.%Y", "%m/%d/%Y"];

/// Parse user input into an instant. Accepts:
/// - epoch digits (unit auto-detected from magnitude: s/ms/µs/ns)
/// - date strings in ISO, European, or US shapes ("now" works too)
pub fn parse(input: &str) -> Option<DateTime<Utc>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.eq_ignore_ascii_case("now") {
        return Some(Utc::now());
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return parse_epoch(trimmed);
    }
    for pattern in DATETIME_PATTERNS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, pattern) {
            return local_to_utc(naive);
        }
    }
    for pattern in DATE_PATTERNS {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, pattern) {
            return local_to_utc(date.and_hms_opt(0, 0, 0)?);
        }
    }
    None
}

fn local_to_utc(naive: NaiveDateTime) -> Option<DateTime<Utc>> {
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|local| local.with_timezone(&Utc))
}

/// Digit-count heuristic: 1786098721 is seconds, 13 digits is
/// millis, 16 micros, anything longer nanos.
fn parse_epoch(digits: &str) -> Option<DateTime<Utc>> {
    let n: i64 = digits.parse().ok()?;
    let (secs, nanos) = match digits.len() {
        0..=11 => (n, 0u32),
        12..=14 => (n / 1_000, (n % 1_000) as u32 * 1_000_000),
        15..=17 => (n / 1_000_000, (n % 1_000_000) as u32 * 1_000),
        _ => (n / 1_000_000_000, (n % 1_000_000_000) as u32),
    };
    Utc.timestamp_opt(secs, nanos).single()
}

/// Apply an offset like "+2h30m", "-45m", "+1d" to an instant.
pub fn offset(base: DateTime<Utc>, spec: &str) -> Option<DateTime<Utc>> {
    let trimmed = spec.trim();
    let (sign, body) = match trimmed.chars().next()? {
        '+' => (1i64, &trimmed[1..]),
        '-' => (-1i64, &trimmed[1..]),
        _ => (1i64, trimmed),
    };
    let mut total_secs = 0i64;
    let mut digits = String::new();
    for ch in body.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        let amount: i64 = digits.parse().ok()?;
        digits.clear();
        let unit = match ch {
            's' => 1,
            'm' => 60,
            'h' => 3_600,
            'd' => 86_400,
            'w' => 604_800,
            _ => return None,
        };
        total_secs += amount * unit;
    }
    if !digits.is_empty() {
        return None; // trailing number without a unit
    }
    base.checked_add_signed(chrono::Duration::seconds(sign * total_secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_epoch_seconds_and_millis_by_length() {
        assert_eq!(parse("1785407677").unwrap().timestamp(), 1_785_407_677,);
        let ms = parse("1785407677500").unwrap();
        assert_eq!(ms.timestamp(), 1_785_407_677);
        assert_eq!(ms.timestamp_millis(), 1_785_407_677_500);
    }

    #[test]
    fn parses_micros_and_nanos_by_length() {
        let us = parse("1785407677000001").unwrap();
        assert_eq!(us.timestamp_micros(), 1_785_407_677_000_001);
        let ns = parse("1785407677000000002").unwrap();
        assert_eq!(ns.timestamp(), 1_785_407_677);
    }

    #[test]
    fn parses_iso_and_european_date_strings() {
        let iso = parse("2026-07-30 12:34:56").unwrap();
        let eu = parse("30.07.2026 12:34:56").unwrap();
        assert_eq!(iso, eu);
        assert!(parse("2026-07-30").is_some());
    }

    #[test]
    fn parses_now_and_rejects_garbage() {
        assert!(parse("now").is_some());
        assert!(parse("yesterday-ish").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn offsets_add_and_subtract_compound_units() {
        let base = Utc.timestamp_opt(1_000_000, 0).unwrap();
        assert_eq!(
            offset(base, "+2h30m").unwrap().timestamp(),
            1_000_000 + 9_000,
        );
        assert_eq!(
            offset(base, "-1d").unwrap().timestamp(),
            1_000_000 - 86_400,
        );
        assert!(offset(base, "+2x").is_none());
        assert!(offset(base, "+15").is_none());
    }
}
