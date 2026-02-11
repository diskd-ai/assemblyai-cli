use std::path::{Path, PathBuf};
use std::time::Duration;

pub mod config;
pub mod subtitles;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeechModel {
    Best,
    Nano,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptFormat {
    Text,
    Srt,
    Vtt,
}

#[derive(Debug, Clone)]
pub enum Input {
    LocalPath(PathBuf),
    Url(url::Url),
}

#[derive(Debug, Clone)]
pub enum Output {
    Stdout,
    FilePath(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    AutoDetect,
    NoDetect,
    Fixed { code: String },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CustomSpelling {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone)]
pub struct TranscribeOptions {
    input: Input,
    output: Output,
    format: TranscriptFormat,
    speech_model: SpeechModel,
    language: Language,
    punctuate: bool,
    format_text: bool,
    disfluencies: bool,
    filter_profanity: bool,
    speaker_labels: bool,
    multichannel: bool,
    word_boost: Vec<String>,
    custom_spelling: Vec<CustomSpelling>,
    chars_per_caption: u32,
    speech_threshold: Option<f64>,
    poll_interval: Duration,
    timeout: Duration,
}

impl TranscribeOptions {
    pub fn new(params: TranscribeOptionsParams) -> Result<Self, DomainError> {
        let input = parse_input(&params.input)?;

        let output = match params.output {
            Some(path) => Output::FilePath(path),
            None => Output::Stdout,
        };

        let language = match (params.language_detection, params.language) {
            (true, None) => Language::AutoDetect,
            (true, Some(_)) => return Err(DomainError::LanguageProvidedWithDetection),
            (false, None) => Language::NoDetect,
            (false, Some(code)) => Language::Fixed { code },
        };

        if let Some(value) = params.speech_threshold {
            if !(0.0..=1.0).contains(&value) {
                return Err(DomainError::InvalidSpeechThreshold { value });
            }
        }

        if params.chars_per_caption == 0 {
            return Err(DomainError::InvalidCharsPerCaption);
        }

        let custom_spelling = params
            .custom_spelling
            .into_iter()
            .enumerate()
            .map(|(index, entry)| {
                let from = entry.from.trim().to_string();
                let to = entry.to.trim().to_string();
                if from.is_empty() || to.is_empty() {
                    return Err(DomainError::InvalidCustomSpellingEntry { index });
                }
                Ok(CustomSpelling { from, to })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            input,
            output,
            format: params.format,
            speech_model: params.speech_model,
            language,
            punctuate: params.punctuate,
            format_text: params.format_text,
            disfluencies: params.disfluencies,
            filter_profanity: params.filter_profanity,
            speaker_labels: params.speaker_labels,
            multichannel: params.multichannel,
            word_boost: params.word_boost,
            custom_spelling,
            chars_per_caption: params.chars_per_caption,
            speech_threshold: params.speech_threshold,
            poll_interval: params.poll_interval,
            timeout: params.timeout,
        })
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn format(&self) -> TranscriptFormat {
        self.format
    }

    pub fn speech_model(&self) -> SpeechModel {
        self.speech_model
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn punctuate(&self) -> bool {
        self.punctuate
    }

    pub fn format_text(&self) -> bool {
        self.format_text
    }

    pub fn disfluencies(&self) -> bool {
        self.disfluencies
    }

    pub fn filter_profanity(&self) -> bool {
        self.filter_profanity
    }

    pub fn speaker_labels(&self) -> bool {
        self.speaker_labels
    }

    pub fn multichannel(&self) -> bool {
        self.multichannel
    }

    pub fn word_boost(&self) -> &[String] {
        &self.word_boost
    }

    pub fn custom_spelling(&self) -> &[CustomSpelling] {
        &self.custom_spelling
    }

    pub fn chars_per_caption(&self) -> u32 {
        self.chars_per_caption
    }

    pub fn speech_threshold(&self) -> Option<f64> {
        self.speech_threshold
    }

    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

pub struct TranscribeOptionsParams {
    pub input: String,
    pub format: TranscriptFormat,
    pub output: Option<PathBuf>,
    pub speech_model: SpeechModel,
    pub language_detection: bool,
    pub language: Option<String>,
    pub punctuate: bool,
    pub format_text: bool,
    pub disfluencies: bool,
    pub filter_profanity: bool,
    pub speaker_labels: bool,
    pub multichannel: bool,
    pub speech_threshold: Option<f64>,
    pub chars_per_caption: u32,
    pub word_boost: Vec<String>,
    pub custom_spelling: Vec<CustomSpelling>,
    pub poll_interval: Duration,
    pub timeout: Duration,
}

#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("unsupported extension for local file: {path:?}")]
    UnsupportedExtension { path: PathBuf },

    #[error("invalid input URL: {value}")]
    InvalidUrl { value: String },

    #[error("invalid speech threshold {value}; expected 0.0..=1.0")]
    InvalidSpeechThreshold { value: f64 },

    #[error("chars-per-caption must be greater than 0")]
    InvalidCharsPerCaption,

    #[error("--language is not allowed when language detection is enabled")]
    LanguageProvidedWithDetection,

    #[error("invalid custom spelling entry {value:?}; expected FROM=TO")]
    InvalidCustomSpelling { value: String },

    #[error("invalid custom spelling entry at index {index}; 'from' and 'to' must be non-empty")]
    InvalidCustomSpellingEntry { index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    Audio,
    Video,
    Unknown,
}

pub fn classify_local_media(path: &Path) -> MediaKind {
    let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()) else {
        return MediaKind::Unknown;
    };

    match ext.as_str() {
        "mp3" | "wav" | "flac" | "m4a" | "ogg" => MediaKind::Audio,
        "mp4" | "avi" | "mov" | "mkv" | "webm" => MediaKind::Video,
        _ => MediaKind::Unknown,
    }
}

fn parse_input(value: &str) -> Result<Input, DomainError> {
    if value.starts_with("http://") || value.starts_with("https://") {
        let url = url::Url::parse(value).map_err(|_| DomainError::InvalidUrl {
            value: value.to_string(),
        })?;
        return Ok(Input::Url(url));
    }

    Ok(Input::LocalPath(PathBuf::from(value)))
}

pub fn parse_custom_spelling_kv(value: &str) -> Result<CustomSpelling, DomainError> {
    let (from, to) = value.split_once('=').ok_or_else(|| DomainError::InvalidCustomSpelling {
        value: value.to_string(),
    })?;

    let from = from.trim();
    let to = to.trim();
    if from.is_empty() || to.is_empty() {
        return Err(DomainError::InvalidCustomSpelling {
            value: value.to_string(),
        });
    }

    Ok(CustomSpelling {
        from: from.to_string(),
        to: to.to_string(),
    })
}
