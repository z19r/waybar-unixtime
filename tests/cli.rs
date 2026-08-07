use assert_cmd::Command;

fn bin() -> Command {
    Command::cargo_bin("waybar-unixtime").expect("binary builds")
}

#[test]
fn once_emits_valid_waybar_json() {
    let output = bin().arg("once").assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout)
        .trim()
        .to_string();
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("valid JSON");
    assert!(value["text"].as_str().unwrap().len() >= 10);
    assert_eq!(value["class"], "seconds");
    assert!(value["tooltip"].as_str().unwrap().contains("UTC"));
}

#[test]
fn once_millis_reports_millis_class() {
    let output = bin().args(["once", "--millis"]).assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout)
        .trim()
        .to_string();
    let value: serde_json::Value =
        serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(value["class"], "millis");
    assert!(value["text"].as_str().unwrap().len() >= 13);
}

#[test]
fn copy_prints_bare_digits() {
    let output = bin().arg("copy").assert().success();
    let stdout = String::from_utf8_lossy(&output.get_output().stdout)
        .trim()
        .to_string();
    assert!(stdout.chars().all(|c| c.is_ascii_digit()));
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

    let output = bin()
        .arg("css")
        .env("OMARCHY_THEME_DIR", &dir)
        .assert()
        .success();
    let stdout =
        String::from_utf8_lossy(&output.get_output().stdout).to_string();
    assert!(stdout.contains("@define-color unixtime-accent #ff00ff;"));
    assert!(stdout.contains("@define-color unixtime-bg #101010;"));
}
