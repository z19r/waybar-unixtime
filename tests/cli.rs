use assert_cmd::Command;

fn bin() -> Command {
    let mut cmd = Command::cargo_bin("waybar-unixtime").expect("binary builds");
    cmd.env(
        "WAYBAR_UNIXTIME_STATE_DIR",
        std::env::temp_dir().join("wut-cli-test-state"),
    );
    cmd
}

fn stdout(cmd: &mut Command) -> String {
    let output = cmd.assert().success();
    String::from_utf8_lossy(&output.get_output().stdout).to_string()
}

#[test]
fn once_emits_valid_waybar_json_with_full_tooltip() {
    let out = stdout(bin().arg("once"));
    let value: serde_json::Value =
        serde_json::from_str(out.trim()).expect("valid JSON");
    assert!(value["text"].as_str().unwrap().len() >= 10);
    let tooltip = value["tooltip"].as_str().unwrap();
    assert!(tooltip.contains("TIMESTAMP"));
    assert!(tooltip.contains("RFC 2822"));
}

#[test]
fn copy_with_format_renders_that_format() {
    let out = stdout(bin().args(["copy", "millis"]));
    let trimmed = out.trim();
    assert!(trimmed.chars().all(|c| c.is_ascii_digit()));
    assert!(trimmed.len() >= 13);

    let iso = stdout(bin().args(["copy", "iso-utc"]));
    assert!(iso.trim().ends_with('Z'));
}

#[test]
fn copy_rejects_unknown_format() {
    bin().args(["copy", "klingon"]).assert().failure();
}

#[test]
fn set_then_once_uses_persisted_format() {
    let dir = std::env::temp_dir().join("wut-cli-test-set");
    let mut set = Command::cargo_bin("waybar-unixtime").unwrap();
    set.env("WAYBAR_UNIXTIME_STATE_DIR", &dir)
        .args(["set", "nanos"])
        .assert()
        .success();

    let mut once = Command::cargo_bin("waybar-unixtime").unwrap();
    once.env("WAYBAR_UNIXTIME_STATE_DIR", &dir).arg("once");
    let out = stdout(&mut once);
    let value: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert_eq!(value["class"], "nanos");
    assert!(value["text"].as_str().unwrap().len() >= 19);
}

#[test]
fn menu_xml_contains_every_format_item() {
    let out = stdout(bin().arg("menu"));
    assert!(out.contains("GtkMenu"));
    assert!(out.contains("id=\"seconds\""));
    assert!(out.contains("id=\"rfc2822\""));
    assert!(out.contains("id=\"japanese\""));
}

#[test]
fn snippet_is_pasteable_waybar_config() {
    let out = stdout(bin().arg("snippet"));
    assert!(out.contains("\"custom/unixtime\""));
    assert!(out.contains("\"menu-actions\""));
    assert!(out.contains("copy iso-utc | wl-copy"));
    assert!(out.contains("\"interval\": 1"));
}

#[test]
fn formats_lists_all_keys() {
    let out = stdout(bin().arg("formats"));
    for key in ["seconds", "nanos", "iso-utc", "japanese", "custom:"] {
        assert!(out.contains(key), "missing {key}");
    }
}

#[test]
fn css_reads_palette_from_omarchy_theme_dir() {
    let dir = std::env::temp_dir().join("waybar-unixtime-test-theme");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("colors.toml"),
        "accent = \"#ff00ff\"\nforeground = \"#eeeeee\"\n\
         background = \"#101010\"\n",
    )
    .unwrap();

    let out = stdout(bin().arg("css").env("OMARCHY_THEME_DIR", &dir));
    assert!(out.contains("@define-color unixtime-accent #ff00ff;"));
    assert!(out.contains("#custom-unixtime.micros"));
}
