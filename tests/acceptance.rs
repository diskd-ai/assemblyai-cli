use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;

fn set_temp_home(cmd: &mut Command) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    cmd.env("HOME", dir.path());
    cmd.env("USERPROFILE", dir.path());
    dir
}

fn config_path(home: &tempfile::TempDir) -> PathBuf {
    home.path().join(".assemblyai-cli")
}

fn config_json_path(home: &tempfile::TempDir) -> PathBuf {
    config_path(home).join("config.json")
}

fn dummy_audio_path() -> &'static str {
    "input.mp3"
}

#[test]
fn init_creates_config_json() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    cmd.arg("init").write_stdin("dummy-key\n");
    cmd.assert().success();

    let path = home.path().join(".assemblyai-cli").join("config.json");
    let contents = std::fs::read_to_string(path).expect("read config.json");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("parse json");
    assert_eq!(
        parsed.get("apiKey").and_then(|v| v.as_str()),
        Some("dummy-key")
    );
}

#[test]
fn init_updates_existing_config_preserving_fields() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);

    let config_dir = home.path().join(".assemblyai-cli");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    let config_path = config_dir.join("config.json");
    std::fs::write(&config_path, r#"{"format":"vtt","timeoutSeconds":123}"#).expect("write config");

    cmd.arg("init").write_stdin("new-key\n");
    cmd.assert().success();

    let contents = std::fs::read_to_string(&config_path).expect("read config.json");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("parse json");
    assert_eq!(parsed.get("format").and_then(|v| v.as_str()), Some("vtt"));
    assert_eq!(
        parsed.get("timeoutSeconds").and_then(|v| v.as_u64()),
        Some(123)
    );
    assert_eq!(parsed.get("apiKey").and_then(|v| v.as_str()), Some("new-key"));
}

#[test]
fn init_existing_api_key_decline_preserves_value() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);

    let config_dir = home.path().join(".assemblyai-cli");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    let config_path = config_dir.join("config.json");
    std::fs::write(&config_path, r#"{"apiKey":"old-key","format":"text"}"#).expect("write config");

    cmd.arg("init").write_stdin("n\n");
    cmd.assert().success();

    let contents = std::fs::read_to_string(&config_path).expect("read config.json");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("parse json");
    assert_eq!(parsed.get("apiKey").and_then(|v| v.as_str()), Some("old-key"));
}

#[test]
fn init_existing_api_key_overwrite_updates_value() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);

    let config_dir = home.path().join(".assemblyai-cli");
    std::fs::create_dir_all(&config_dir).expect("create config dir");
    let config_path = config_dir.join("config.json");
    std::fs::write(&config_path, r#"{"apiKey":"old-key","format":"vtt"}"#).expect("write config");

    cmd.arg("init").write_stdin("y\nnew-key\n");
    cmd.assert().success();

    let contents = std::fs::read_to_string(&config_path).expect("read config.json");
    let parsed: serde_json::Value = serde_json::from_str(&contents).expect("parse json");
    assert_eq!(parsed.get("format").and_then(|v| v.as_str()), Some("vtt"));
    assert_eq!(parsed.get("apiKey").and_then(|v| v.as_str()), Some("new-key"));
}

#[test]
fn help_mentions_config_and_env_vars() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    cmd.arg("--help");
    cmd.assert().success().stdout(
        predicate::str::contains("~/.assemblyai-cli/config.json")
            .and(predicate::str::contains("ASSEMBLYAI_API_KEY"))
            .and(predicate::str::contains("ASSEMBLY_AI_KEY")),
    );
}

#[test]
fn transcribe_help_mentions_formats_and_diarization() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    cmd.arg("transcribe").arg("--help");
    cmd.assert().success().stdout(
        predicate::str::contains("--format")
            .and(predicate::str::contains("srt"))
            .and(predicate::str::contains("vtt"))
            .and(predicate::str::contains("--speaker-labels"))
            .and(predicate::str::contains("ffmpeg")),
    );
}

#[test]
fn missing_api_key_exits_3() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let _home = set_temp_home(&mut cmd);
    cmd.arg("transcribe").arg(dummy_audio_path());
    cmd.env_remove("ASSEMBLYAI_API_KEY");
    cmd.env_remove("ASSEMBLY_AI_KEY");
    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("ASSEMBLYAI_API_KEY"));
}

#[test]
fn unsupported_extension_exits_2() {
    let tmp = tempfile::Builder::new()
        .prefix("assemblyai-cli-")
        .suffix(".txt")
        .tempfile()
        .expect("tempfile");
    let path = tmp.path().to_path_buf();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let _home = set_temp_home(&mut cmd);
    cmd.env("ASSEMBLYAI_API_KEY", "dummy");
    cmd.arg("transcribe").arg(path);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("unsupported extension"));
}

#[test]
fn invalid_config_json_exits_3() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    std::fs::write(config_path(&home), "{ not-json").expect("write config");

    cmd.env_remove("ASSEMBLYAI_API_KEY");
    cmd.env_remove("ASSEMBLY_AI_KEY");
    cmd.arg("transcribe").arg(dummy_audio_path());
    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("failed to parse config file"));
}

#[test]
fn config_path_is_directory_exits_3() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    std::fs::create_dir_all(config_json_path(&home)).expect("create config dir");

    cmd.env_remove("ASSEMBLYAI_API_KEY");
    cmd.env_remove("ASSEMBLY_AI_KEY");
    cmd.arg("transcribe").arg(dummy_audio_path());
    cmd.assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("failed to read config file"));
}

#[test]
fn invalid_custom_spelling_in_config_exits_2() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    let json = r#"{"customSpelling":[{"from":"","to":"x"}]}"#;
    std::fs::write(config_path(&home), json).expect("write config");

    cmd.env("ASSEMBLYAI_API_KEY", "dummy");
    cmd.arg("transcribe").arg(dummy_audio_path());
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid custom spelling entry"));
}

#[test]
fn invalid_speech_threshold_in_config_exits_2() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    let json = r#"{"speechThreshold":1.5}"#;
    std::fs::write(config_path(&home), json).expect("write config");

    cmd.env("ASSEMBLYAI_API_KEY", "dummy");
    cmd.arg("transcribe").arg(dummy_audio_path());
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid speech threshold"));
}
