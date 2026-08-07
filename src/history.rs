use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// One copied timestamp — the "History" tab.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    /// Epoch seconds when the copy happened.
    pub at: i64,
    /// Format key that was copied.
    pub format: String,
    /// The copied value.
    pub value: String,
}

fn history_file() -> Option<PathBuf> {
    if let Ok(dir) = env::var("WAYBAR_UNIXTIME_STATE_DIR") {
        return Some(PathBuf::from(dir).join("history.jsonl"));
    }
    let base = match env::var_os("XDG_STATE_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(env::var_os("HOME")?).join(".local/state"),
    };
    Some(base.join("waybar-unixtime/history.jsonl"))
}

/// Append an entry, trimming the file to `cap` newest entries.
/// A cap of 0 disables history entirely.
pub fn record(entry: &Entry, cap: usize) -> std::io::Result<()> {
    if cap == 0 {
        return Ok(());
    }
    let Some(path) = history_file() else {
        return Err(std::io::Error::other("cannot resolve state dir"));
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut entries = read_all();
    entries.push(entry.clone());
    let skip = entries.len().saturating_sub(cap);
    let body: String = entries
        .iter()
        .skip(skip)
        .filter_map(|e| serde_json::to_string(e).ok())
        .map(|line| line + "\n")
        .collect();
    fs::write(path, body)
}

/// Newest-first list of recorded entries.
pub fn list(limit: usize) -> Vec<Entry> {
    let mut entries = read_all();
    entries.reverse();
    entries.truncate(limit);
    entries
}

fn read_all() -> Vec<Entry> {
    let Some(path) = history_file() else {
        return Vec::new();
    };
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(n: i64) -> Entry {
        Entry {
            at: n,
            format: String::from("seconds"),
            value: n.to_string(),
        }
    }

    // Serial: the env var is process-global.
    #[test]
    fn records_caps_and_lists_newest_first() {
        let _guard = crate::state::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = std::env::temp_dir().join("wut-history-test");
        let _ = std::fs::remove_dir_all(&dir);
        env::set_var("WAYBAR_UNIXTIME_STATE_DIR", &dir);

        for n in 0..5 {
            record(&entry(n), 3).unwrap();
        }
        let listed = list(10);
        assert_eq!(listed.len(), 3);
        assert_eq!(listed[0].at, 4);
        assert_eq!(listed[2].at, 2);

        // cap 0 disables recording
        let dir2 = std::env::temp_dir().join("wut-history-off");
        let _ = std::fs::remove_dir_all(&dir2);
        env::set_var("WAYBAR_UNIXTIME_STATE_DIR", &dir2);
        record(&entry(9), 0).unwrap();
        assert!(list(10).is_empty());

        env::remove_var("WAYBAR_UNIXTIME_STATE_DIR");
    }
}
