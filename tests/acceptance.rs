use std::path::{Path, PathBuf};

use assert_cmd::Command;
use base64::Engine;
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

fn load_api_key_from_dotenv() -> Option<String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let env_path = manifest_dir.join(".env");
    let contents = std::fs::read_to_string(env_path).ok()?;

    let mut direct: Option<String> = None;
    let mut encoded: Option<String> = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line.split_once('=')?;
        let key = key.trim();
        let mut value = value.trim().to_string();
        if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
            value = value[1..value.len().saturating_sub(1)].to_string();
        }

        if key == "ASSEMBLYAI_API_KEY" && !value.trim().is_empty() {
            direct = Some(value);
            break;
        }

        if key == "ASSEMBLY_AI_KEY" && !value.trim().is_empty() {
            encoded = Some(value);
        }
    }

    if let Some(value) = direct {
        return Some(value);
    }

    let encoded = encoded?;
    let padded = {
        let mut value = encoded.trim().to_string();
        while value.len() % 4 != 0 {
            value.push('=');
        }
        value
    };

    let decoded = base64::engine::general_purpose::STANDARD.decode(padded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let decoded = decoded.trim().to_string();
    if decoded.is_empty() { None } else { Some(decoded) }
}

fn demo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
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
    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
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
fn transcribes_demo_mp3_to_stdout() {
    let Some(api_key) = load_api_key_from_dotenv() else {
        eprintln!("skipping: missing ASSEMBLYAI_API_KEY/ASSEMBLY_AI_KEY");
        return;
    };

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let _home = set_temp_home(&mut cmd);
    cmd.env("ASSEMBLYAI_API_KEY", api_key);
    cmd.arg("transcribe")
        .arg(demo_path("demo/part3.mp3"))
        .arg("--timeout-seconds")
        .arg("900");

    cmd.assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn transcribes_demo_mp3_with_speaker_labels() {
    let Some(api_key) = load_api_key_from_dotenv() else {
        eprintln!("skipping: missing ASSEMBLYAI_API_KEY/ASSEMBLY_AI_KEY");
        return;
    };

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let _home = set_temp_home(&mut cmd);
    cmd.env("ASSEMBLYAI_API_KEY", api_key);
    cmd.arg("transcribe")
        .arg(demo_path("demo/part3.mp3"))
        .arg("--speaker-labels")
        .arg("--timeout-seconds")
        .arg("900");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Speaker").and(predicate::str::is_empty().not()));
}

#[test]
fn transcribes_demo_mp4_to_srt_file() {
    let Some(api_key) = load_api_key_from_dotenv() else {
        eprintln!("skipping: missing ASSEMBLYAI_API_KEY/ASSEMBLY_AI_KEY");
        return;
    };

    let out_dir = tempfile::tempdir().expect("tempdir");
    let out_path = out_dir.path().join("part3.srt");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let _home = set_temp_home(&mut cmd);
    cmd.env("ASSEMBLYAI_API_KEY", api_key);
    cmd.arg("transcribe")
        .arg(demo_path("demo/part3.mp4"))
        .arg("--format")
        .arg("srt")
        .arg("--output")
        .arg(&out_path)
        .arg("--chars-per-caption")
        .arg("256")
        .arg("--timeout-seconds")
        .arg("900");

    cmd.assert().success();

    let content = std::fs::read_to_string(&out_path).expect("read srt");
    assert!(!content.trim().is_empty(), "srt is non-empty");
    assert!(content.contains("-->"), "srt contains time ranges");
}

#[test]
fn config_file_all_keys_diarized_vtt() {
    let Some(api_key) = load_api_key_from_dotenv() else {
        eprintln!("skipping: missing ASSEMBLYAI_API_KEY/ASSEMBLY_AI_KEY");
        return;
    };

    let home_dir = tempfile::tempdir().expect("tempdir");
    let output_path = home_dir.path().join("config-output.vtt");
    let config_dir = home_dir.path().join(".assemblyai-cli");
    let config_path = config_dir.join("config.json");
    std::fs::create_dir_all(&config_dir).expect("create config dir");

    let config_json = serde_json::json!({
        "apiKey": api_key,
        "baseUrl": "https://api.assemblyai.com",
        "format": "vtt",
        "output": output_path.to_str().expect("utf-8 path"),
        "speechModel": "best",
        "languageDetection": false,
        "language": "ru",
        "punctuate": true,
        "formatText": true,
        "disfluencies": false,
        "filterProfanity": false,
        "speakerLabels": true,
        "multichannel": false,
        "speechThreshold": 0.1,
        "charsPerCaption": 128,
        "wordBoost": ["Клод"],
        "customSpelling": [{"from":"Клод","to":"Claude"}],
        "pollIntervalSeconds": 1,
        "timeoutSeconds": 900
    });
    std::fs::write(&config_path, config_json.to_string()).expect("write config file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    cmd.env("HOME", home_dir.path());
    cmd.env("USERPROFILE", home_dir.path());
    cmd.env_remove("ASSEMBLYAI_API_KEY");
    cmd.env_remove("ASSEMBLY_AI_KEY");

    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
    cmd.assert().success();

    let content = std::fs::read_to_string(&output_path).expect("read output");
    assert!(!content.trim().is_empty(), "config output is non-empty");
    assert!(content.starts_with("WEBVTT\n\n"), "config output is vtt");
    assert!(content.contains("Speaker "), "config output includes diarization");
    assert!(
        content.contains("Claude"),
        "config output applies custom spelling (expected 'Claude')"
    );
}

#[test]
fn invalid_config_json_exits_3() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("assemblyai-cli"));
    let home = set_temp_home(&mut cmd);
    std::fs::write(config_path(&home), "{ not-json").expect("write config");

    cmd.env_remove("ASSEMBLYAI_API_KEY");
    cmd.env_remove("ASSEMBLY_AI_KEY");
    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
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
    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
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
    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
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
    cmd.arg("transcribe").arg(demo_path("demo/part3.mp3"));
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("invalid speech threshold"));
}
