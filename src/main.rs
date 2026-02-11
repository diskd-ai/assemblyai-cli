mod app;
mod domain;
mod infra;

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use base64::Engine;
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::domain::{CustomSpelling, TranscriptFormat, TranscribeOptions};

#[derive(Parser, Debug)]
#[command(
    name = "assemblyai-cli",
    version,
    about = "Transcribe audio/video files using AssemblyAI",
    long_about = "Transcribe a single local file or URL using AssemblyAI.\n\nUse `assemblyai-cli init` to create a config file.\nUse `assemblyai-cli transcribe --help` for transcription options.\n",
    after_help = r#"CONFIG
  ~/.assemblyai-cli/config.json (preferred) or ~/.assemblyai-cli (legacy)

API KEY RESOLUTION ORDER
  1. config apiKey
  2. ASSEMBLYAI_API_KEY
  3. ASSEMBLY_AI_KEY (base64-encoded; decoded automatically if it looks like base64)

ENV VARS
  ASSEMBLYAI_BASE_URL (optional; default https://api.assemblyai.com)

EXAMPLES
  assemblyai-cli init
  assemblyai-cli transcribe demo/part3.mp3
  assemblyai-cli transcribe demo/part3.mp4 --format srt --output part3.srt
  assemblyai-cli transcribe demo/part3.mp3 --speaker-labels
  assemblyai-cli transcribe https://example.com/audio.wav --format vtt
"#
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(
        about = "Transcribe a single file or URL",
        long_about = "Transcribe a single local audio/video file or HTTP(S) URL.\n\nFor local video files, ffmpeg must be available on PATH so the CLI can extract audio.\n"
    )]
    Transcribe(TranscribeArgs),

    #[command(
        about = "Initialize ~/.assemblyai-cli config",
        long_about = "Interactively initialize the AssemblyAI CLI configuration.\n\nThis command prompts for an API key and writes it to ~/.assemblyai-cli/config.json (preferred) or ~/.assemblyai-cli (legacy).\n"
    )]
    Init(InitArgs),
}

#[derive(Clone, Debug, ValueEnum)]
enum FormatArg {
    Text,
    Srt,
    Vtt,
}

impl From<FormatArg> for TranscriptFormat {
    fn from(value: FormatArg) -> Self {
        match value {
            FormatArg::Text => TranscriptFormat::Text,
            FormatArg::Srt => TranscriptFormat::Srt,
            FormatArg::Vtt => TranscriptFormat::Vtt,
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
enum SpeechModelArg {
    Best,
    Nano,
}

impl From<SpeechModelArg> for domain::SpeechModel {
    fn from(value: SpeechModelArg) -> Self {
        match value {
            SpeechModelArg::Best => domain::SpeechModel::Best,
            SpeechModelArg::Nano => domain::SpeechModel::Nano,
        }
    }
}

#[derive(Args, Debug)]
#[command(
    after_help = r#"INPUT
  INPUT can be a local file path (audio/video) or an HTTP(S) URL.

LOCAL FILES
  Audio extensions: mp3, wav, flac, m4a, ogg
  Video extensions: mp4, avi, mov, mkv, webm (requires ffmpeg)

OUTPUT
  If --output is omitted, the transcript is printed to stdout.
  If --output is provided, the transcript is written to the file and a status line is printed to stderr.

DIARIZATION (SPEAKER LABELS)
  --speaker-labels enables speaker diarization when the API provides utterances.
  - text: prints "Speaker X: ..." lines
  - srt/vtt: prefers diarized subtitles ("Speaker X: ...") when possible

CONFIG
  ~/.assemblyai-cli/config.json (preferred) or ~/.assemblyai-cli (legacy)
  CLI flags override config values.
"#
)]
struct TranscribeArgs {
    #[arg(
        value_name = "INPUT",
        help = "Local audio/video path or HTTP(S) URL"
    )]
    input: String,

    #[arg(
        long,
        value_enum,
        help = "Output format (text, srt, vtt); when omitted, uses config `format` or defaults to text"
    )]
    format: Option<FormatArg>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Write transcript to PATH; when omitted, uses config `output` or stdout"
    )]
    output: Option<PathBuf>,

    #[arg(
        long,
        value_enum,
        help = "Speech model (best, nano); when omitted, uses config `speechModel` or defaults to best"
    )]
    speech_model: Option<SpeechModelArg>,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "no_language_detection",
        help = "Enable language detection; when omitted, uses config `languageDetection` or defaults to enabled"
    )]
    language_detection: bool,

    #[arg(
        long = "no-language-detection",
        action = clap::ArgAction::SetTrue,
        help = "Disable language detection; only valid with --language to set the language code"
    )]
    no_language_detection: bool,

    #[arg(
        long,
        value_name = "CODE",
        help = "Language code (e.g. en, ru); only valid when language detection is disabled"
    )]
    language: Option<String>,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "no_punctuate",
        help = "Enable punctuation; when omitted, uses config `punctuate` or defaults to enabled"
    )]
    punctuate: bool,

    #[arg(
        long = "no-punctuate",
        action = clap::ArgAction::SetTrue,
        help = "Disable punctuation"
    )]
    no_punctuate: bool,

    #[arg(
        long = "format-text",
        action = clap::ArgAction::SetTrue,
        conflicts_with = "no_format_text",
        help = "Enable text formatting; when omitted, uses config `formatText` or defaults to enabled"
    )]
    format_text: bool,

    #[arg(
        long = "no-format-text",
        action = clap::ArgAction::SetTrue,
        help = "Disable text formatting"
    )]
    no_format_text: bool,

    #[arg(long, help = "Include disfluencies (filler words); when omitted, uses config `disfluencies` or defaults to disabled")]
    disfluencies: bool,

    #[arg(long = "filter-profanity", help = "Filter profanity; when omitted, uses config `filterProfanity` or defaults to disabled")]
    filter_profanity: bool,

    #[arg(long = "speaker-labels", help = "Enable speaker diarization (speaker labels); when omitted, uses config `speakerLabels` or defaults to disabled")]
    speaker_labels: bool,

    #[arg(
        long,
        action = clap::ArgAction::SetTrue,
        conflicts_with = "no_multichannel",
        help = "Enable multichannel audio; when omitted, uses config `multichannel` or defaults to enabled"
    )]
    multichannel: bool,

    #[arg(
        long = "no-multichannel",
        action = clap::ArgAction::SetTrue,
        help = "Disable multichannel audio"
    )]
    no_multichannel: bool,

    #[arg(long, value_name = "0.0..1.0", help = "Speech threshold (0.0..=1.0); when omitted, uses config `speechThreshold`")]
    speech_threshold: Option<f64>,

    #[arg(long, value_name = "N", help = "Max characters per caption for srt/vtt; when omitted, uses config `charsPerCaption` or defaults to 128")]
    chars_per_caption: Option<u32>,

    #[arg(long = "word-boost", value_name = "PHRASE", help = "Boost recognition for PHRASE (repeatable); when omitted, uses config `wordBoost`")]
    word_boost: Vec<String>,

    #[arg(long = "custom-spelling", value_name = "FROM=TO", help = "Custom spelling mapping (repeatable); when omitted, uses config `customSpelling`")]
    custom_spelling: Vec<String>,

    #[arg(long, value_name = "SECONDS", help = "Polling interval (seconds); when omitted, uses config `pollIntervalSeconds` or defaults to 3")]
    poll_interval_seconds: Option<u64>,

    #[arg(long, value_name = "SECONDS", help = "Timeout (seconds); when omitted, uses config `timeoutSeconds` or defaults to 3600")]
    timeout_seconds: Option<u64>,
}

#[derive(Args, Debug)]
#[command(
    after_help = r#"CONFIG PATHS
  ~/.assemblyai-cli/config.json (preferred) or ~/.assemblyai-cli (legacy)

BEHAVIOR
  - Prompts for an API key on stdin and writes config.
  - If a config file already exists and is valid JSON, it preserves all existing fields and only updates `apiKey`.
  - If `apiKey` already exists, it asks before overwriting unless --yes is provided.
  - Use --force to overwrite an invalid config.

EXAMPLES
  assemblyai-cli init
  assemblyai-cli init --force
  assemblyai-cli init --yes
"#
)]
struct InitArgs {
    #[arg(long, help = "Overwrite config even if it is invalid JSON")]
    force: bool,

    #[arg(long, help = "Overwrite existing apiKey without prompting")]
    yes: bool,
}

#[derive(thiserror::Error, Debug)]
enum RunError {
    #[error("missing AssemblyAI API key (set ASSEMBLYAI_API_KEY or put apiKey in ~/.assemblyai-cli/config.json)")]
    MissingApiKey,

    #[error("unable to determine home directory (HOME/USERPROFILE is not set)")]
    HomeNotFound,

    #[error("failed to read config file {path:?}: {message}")]
    ConfigRead { path: PathBuf, message: String },

    #[error("failed to parse config file {path:?}: {message}")]
    ConfigParse { path: PathBuf, message: String },

    #[error("failed to write config file {path:?}: {message}")]
    ConfigWrite { path: PathBuf, message: String },

    #[error("failed to read API key from stdin: {message}")]
    InitReadStdin { message: String },

    #[error("API key cannot be empty")]
    InitEmptyApiKey,

    #[error(transparent)]
    Domain(#[from] domain::DomainError),

    #[error(transparent)]
    Infra(#[from] infra::InfraError),

    #[error(transparent)]
    Api(#[from] infra::assemblyai::ApiError),
}

impl From<infra::runner::RunnerError> for RunError {
    fn from(value: infra::runner::RunnerError) -> Self {
        match value {
            infra::runner::RunnerError::Infra(err) => RunError::Infra(err),
            infra::runner::RunnerError::Api(err) => RunError::Api(err),
        }
    }
}

impl RunError {
    fn exit_code(&self) -> u8 {
        match self {
            RunError::Domain(_) => 2,
            RunError::MissingApiKey => 3,
            RunError::HomeNotFound => 3,
            RunError::ConfigRead { .. }
            | RunError::ConfigParse { .. }
            | RunError::ConfigWrite { .. }
            | RunError::InitReadStdin { .. }
            | RunError::InitEmptyApiKey => 3,
            RunError::Infra(err) => err.exit_code(),
            RunError::Api(_) => 5,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Transcribe(args) => run_transcribe(args).await,
        Commands::Init(args) => run_init(args),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(err.exit_code())
        }
    }
}

async fn run_transcribe(args: TranscribeArgs) -> Result<(), RunError> {
    let config = load_config_file()?;
    let api_key = load_api_key(config.as_ref())?;

    let base_url = std::env::var("ASSEMBLYAI_BASE_URL")
        .ok()
        .or_else(|| config.as_ref().and_then(|c| c.base_url.clone()));

    let format = args
        .format
        .map(Into::into)
        .or_else(|| config.as_ref().and_then(|c| c.format))
        .unwrap_or(TranscriptFormat::Text);

    let output = args.output.or_else(|| config.as_ref().and_then(|c| c.output.clone()));

    let speech_model = args
        .speech_model
        .map(Into::into)
        .or_else(|| config.as_ref().and_then(|c| c.speech_model))
        .unwrap_or(domain::SpeechModel::Best);

    let language_detection = match (
        cli_bool_override(args.language_detection, args.no_language_detection),
        config.as_ref().and_then(|c| c.language_detection),
    ) {
        (Some(value), _) => value,
        (None, Some(value)) => value,
        (None, None) => true,
    };

    let language = args.language.or_else(|| config.as_ref().and_then(|c| c.language.clone()));

    let punctuate = match (
        cli_bool_override(args.punctuate, args.no_punctuate),
        config.as_ref().and_then(|c| c.punctuate),
    ) {
        (Some(value), _) => value,
        (None, Some(value)) => value,
        (None, None) => true,
    };

    let format_text = match (
        cli_bool_override(args.format_text, args.no_format_text),
        config.as_ref().and_then(|c| c.format_text),
    ) {
        (Some(value), _) => value,
        (None, Some(value)) => value,
        (None, None) => true,
    };

    let multichannel = match (
        cli_bool_override(args.multichannel, args.no_multichannel),
        config.as_ref().and_then(|c| c.multichannel),
    ) {
        (Some(value), _) => value,
        (None, Some(value)) => value,
        (None, None) => true,
    };

    let disfluencies = if args.disfluencies {
        true
    } else {
        config.as_ref().and_then(|c| c.disfluencies).unwrap_or(false)
    };

    let filter_profanity = if args.filter_profanity {
        true
    } else {
        config
            .as_ref()
            .and_then(|c| c.filter_profanity)
            .unwrap_or(false)
    };

    let speaker_labels = if args.speaker_labels {
        true
    } else {
        config
            .as_ref()
            .and_then(|c| c.speaker_labels)
            .unwrap_or(false)
    };

    let speech_threshold = args
        .speech_threshold
        .or_else(|| config.as_ref().and_then(|c| c.speech_threshold));

    let chars_per_caption = args
        .chars_per_caption
        .or_else(|| config.as_ref().and_then(|c| c.chars_per_caption))
        .unwrap_or(128);

    let word_boost = if args.word_boost.is_empty() {
        config
            .as_ref()
            .and_then(|c| c.word_boost.clone())
            .unwrap_or_default()
    } else {
        args.word_boost
    };

    let custom_spelling_cli = args
        .custom_spelling
        .into_iter()
        .map(|s| domain::parse_custom_spelling_kv(&s))
        .collect::<Result<Vec<CustomSpelling>, domain::DomainError>>()?;

    let custom_spelling = if custom_spelling_cli.is_empty() {
        config
            .as_ref()
            .and_then(|c| c.custom_spelling.clone())
            .unwrap_or_default()
    } else {
        custom_spelling_cli
    };

    let poll_interval_seconds = args
        .poll_interval_seconds
        .or_else(|| config.as_ref().and_then(|c| c.poll_interval_seconds))
        .unwrap_or(3);

    let timeout_seconds = args
        .timeout_seconds
        .or_else(|| config.as_ref().and_then(|c| c.timeout_seconds))
        .unwrap_or(3600);

    let options = TranscribeOptions::new(domain::TranscribeOptionsParams {
        input: args.input,
        format,
        output,
        speech_model,
        language_detection,
        language,
        punctuate,
        format_text,
        disfluencies,
        filter_profanity,
        speaker_labels,
        multichannel,
        speech_threshold,
        chars_per_caption,
        word_boost,
        custom_spelling,
        poll_interval: Duration::from_secs(poll_interval_seconds),
        timeout: Duration::from_secs(timeout_seconds),
    })?;

    let plan = app::build_plan(&options)?;

    let client = infra::assemblyai::AssemblyAiClient::new(infra::assemblyai::AssemblyAiClientConfig {
        api_key,
        base_url,
    })?;

    infra::runner::run_transcribe(plan, client, &options).await?;
    Ok(())
}

fn run_init(args: InitArgs) -> Result<(), RunError> {
    let Some(root_path) = default_config_path() else {
        return Err(RunError::HomeNotFound);
    };

    let target_path = init_target_config_path(&root_path).map_err(|message| RunError::ConfigWrite {
        path: root_path.clone(),
        message,
    })?;

    let mut config_value = if target_path.exists() {
        let contents = std::fs::read_to_string(&target_path).map_err(|err| RunError::ConfigRead {
            path: target_path.clone(),
            message: err.to_string(),
        })?;

        match serde_json::from_str::<serde_json::Value>(&contents) {
            Ok(value) => value,
            Err(err) => {
                if args.force {
                    serde_json::Value::Object(serde_json::Map::new())
                } else {
                    return Err(RunError::ConfigParse {
                        path: target_path.clone(),
                        message: err.to_string(),
                    });
                }
            }
        }
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };

    let is_object = config_value.is_object();
    if !is_object {
        if args.force {
            config_value = serde_json::Value::Object(serde_json::Map::new());
        } else {
            return Err(RunError::ConfigParse {
                path: target_path.clone(),
                message: "config file must be a JSON object".to_string(),
            });
        }
    }

    let Some(obj) = config_value.as_object_mut() else {
        return Err(RunError::ConfigParse {
            path: target_path.clone(),
            message: "config file must be a JSON object".to_string(),
        });
    };

    let existing_api_key = obj
        .get("apiKey")
        .and_then(|v| v.as_str())
        .and_then(non_empty_trimmed);
    if existing_api_key.is_some() && !args.force && !args.yes {
        let overwrite = prompt_overwrite_existing_api_key(&target_path)?;
        if !overwrite {
            eprintln!("init aborted; existing apiKey preserved (use --yes to overwrite)");
            return Ok(());
        }
    }

    let api_key = prompt_api_key_from_stdin()?;
    let api_key = normalize_api_key(&api_key);
    obj.insert("apiKey".to_string(), serde_json::Value::String(api_key));

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| RunError::ConfigWrite {
            path: parent.to_path_buf(),
            message: err.to_string(),
        })?;
    }

    let serialized =
        serde_json::to_string_pretty(&config_value).map_err(|err| RunError::ConfigWrite {
            path: target_path.clone(),
            message: err.to_string(),
        })?;

    std::fs::write(&target_path, format!("{serialized}\n")).map_err(|err| RunError::ConfigWrite {
        path: target_path.clone(),
        message: err.to_string(),
    })?;

    eprintln!("wrote config to {}", target_path.display());
    Ok(())
}

fn init_target_config_path(root_path: &std::path::Path) -> Result<PathBuf, String> {
    if root_path.exists() {
        if root_path.is_dir() {
            return Ok(root_path.join("config.json"));
        }

        if root_path.is_file() {
            return Ok(root_path.to_path_buf());
        }

        return Err("config path exists but is neither a file nor a directory".to_string());
    }

    std::fs::create_dir_all(root_path).map_err(|err| err.to_string())?;
    Ok(root_path.join("config.json"))
}

fn prompt_api_key_from_stdin() -> Result<String, RunError> {
    use std::io::Write;

    eprint!("AssemblyAI API key: ");
    let _ = std::io::stderr().flush();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|err| RunError::InitReadStdin {
            message: err.to_string(),
        })?;

    let mut value = input.trim().to_string();
    if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
        value = value[1..value.len().saturating_sub(1)].to_string();
    }

    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(RunError::InitEmptyApiKey);
    }

    Ok(value)
}

fn prompt_overwrite_existing_api_key(path: &std::path::Path) -> Result<bool, RunError> {
    use std::io::Write;

    eprint!("Config already exists at {}. Overwrite apiKey? [y/N]: ", path.display());
    let _ = std::io::stderr().flush();

    let mut input = String::new();
    let bytes = std::io::stdin()
        .read_line(&mut input)
        .map_err(|err| RunError::InitReadStdin {
            message: err.to_string(),
        })?;

    if bytes == 0 {
        return Ok(false);
    }

    let answer = input.trim().to_ascii_lowercase();
    Ok(answer == "y" || answer == "yes")
}

fn cli_bool_override(yes_flag: bool, no_flag: bool) -> Option<bool> {
    if yes_flag {
        Some(true)
    } else if no_flag {
        Some(false)
    } else {
        None
    }
}

fn load_config_file() -> Result<Option<domain::config::ConfigFile>, RunError> {
    let Some(path) = default_config_path() else {
        return Ok(None);
    };

    let Some(path) = resolve_config_file_path(&path) else {
        return Ok(None);
    };

    let contents = std::fs::read_to_string(&path).map_err(|err| RunError::ConfigRead {
        path: path.clone(),
        message: err.to_string(),
    })?;

    let config: domain::config::ConfigFile =
        serde_json::from_str(&contents).map_err(|err| RunError::ConfigParse {
            path,
            message: err.to_string(),
        })?;

    Ok(Some(config))
}

fn resolve_config_file_path(path: &std::path::Path) -> Option<PathBuf> {
    if !path.exists() {
        return None;
    }

    if path.is_file() {
        return Some(path.to_path_buf());
    }

    if path.is_dir() {
        let candidate = path.join("config.json");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn default_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())?;
    Some(PathBuf::from(home).join(".assemblyai-cli"))
}

fn load_api_key(config: Option<&domain::config::ConfigFile>) -> Result<String, RunError> {
    if let Some(value) = config
        .and_then(|c| c.api_key.as_deref())
        .and_then(non_empty_trimmed)
    {
        return Ok(normalize_api_key(value));
    }

    if let Some(value) = std::env::var("ASSEMBLYAI_API_KEY")
        .ok()
        .as_deref()
        .and_then(non_empty_trimmed)
    {
        return Ok(normalize_api_key(value));
    }

    if let Some(value) = std::env::var("ASSEMBLY_AI_KEY").ok().as_deref().and_then(non_empty_trimmed) {
        if let Some(decoded) = decode_base64_to_hex_key(value) {
            return Ok(decoded);
        }
    }

    Err(RunError::MissingApiKey)
}

fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn normalize_api_key(value: &str) -> String {
    let trimmed = value.trim();
    if looks_like_hex_key(trimmed) {
        return trimmed.to_string();
    }

    if let Some(decoded) = decode_base64_to_hex_key(trimmed) {
        return decoded;
    }

    trimmed.to_string()
}

fn decode_base64_to_hex_key(value: &str) -> Option<String> {
    let mut padded = value.trim().to_string();
    while padded.len() % 4 != 0 {
        padded.push('=');
    }

    let decoded = base64::engine::general_purpose::STANDARD.decode(padded).ok()?;
    let decoded = String::from_utf8(decoded).ok()?;
    let decoded = decoded.trim().to_string();

    if looks_like_hex_key(&decoded) {
        Some(decoded)
    } else {
        None
    }
}

fn looks_like_hex_key(value: &str) -> bool {
    matches!(value.len(), 32 | 64) && value.chars().all(|c| c.is_ascii_hexdigit())
}
