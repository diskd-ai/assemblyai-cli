use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::domain::{CustomSpelling, Language, SpeechModel, TranscriptFormat, TranscribeOptions};

#[derive(Debug, Clone)]
pub struct AssemblyAiClientConfig {
    pub api_key: String,
    pub base_url: Option<String>,
}

#[derive(Clone)]
pub struct AssemblyAiClient {
    base_url: String,
    http: reqwest::Client,
}

impl AssemblyAiClient {
    pub fn new(config: AssemblyAiClientConfig) -> Result<Self, ApiError> {
        let base_url = config
            .base_url
            .unwrap_or_else(|| "https://api.assemblyai.com".to_string());

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str(&config.api_key).map_err(|_| ApiError::InvalidApiKey)?,
        );

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(ApiError::HttpClientBuild)?;

        Ok(Self { base_url, http })
    }

    pub async fn upload_file(&self, path: &std::path::Path) -> Result<String, ApiError> {
        let file = tokio::fs::File::open(path).await.map_err(ApiError::Io)?;
        let stream = tokio_util::io::ReaderStream::new(file);
        let body = reqwest::Body::wrap_stream(stream);

        let url = format!("{}/v2/upload", self.base_url.trim_end_matches('/'));
        let resp = self.http.post(url).body(body).send().await.map_err(ApiError::Http)?;

        let status = resp.status();
        let text = resp.text().await.map_err(ApiError::Http)?;
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status, body: text });
        }

        let parsed: UploadResponse = serde_json::from_str(&text).map_err(ApiError::Json)?;
        Ok(parsed.upload_url)
    }

    pub async fn create_transcript(&self, audio_url: &str, options: &TranscribeOptions) -> Result<Transcript, ApiError> {
        let url = format!("{}/v2/transcript", self.base_url.trim_end_matches('/'));
        let request = CreateTranscriptRequest::from_options(audio_url, options);

        let resp = self
            .http
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(ApiError::Http)?;

        parse_json_response::<Transcript>(resp).await
    }

    pub async fn get_transcript(&self, id: &str) -> Result<Transcript, ApiError> {
        let url = format!("{}/v2/transcript/{}", self.base_url.trim_end_matches('/'), id);
        let resp = self.http.get(url).send().await.map_err(ApiError::Http)?;
        parse_json_response::<Transcript>(resp).await
    }

    pub async fn get_subtitles(
        &self,
        id: &str,
        format: TranscriptFormat,
        chars_per_caption: u32,
    ) -> Result<String, ApiError> {
        let suffix = match format {
            TranscriptFormat::Srt => "srt",
            TranscriptFormat::Vtt => "vtt",
            TranscriptFormat::Text => return Err(ApiError::InvalidSubtitleFormat),
        };

        let url = format!(
            "{}/v2/transcript/{}/{}?chars_per_caption={}",
            self.base_url.trim_end_matches('/'),
            id,
            suffix,
            chars_per_caption
        );

        let resp = self.http.get(url).send().await.map_err(ApiError::Http)?;
        let status = resp.status();
        let body = resp.text().await.map_err(ApiError::Http)?;
        if !status.is_success() {
            return Err(ApiError::HttpStatus { status, body });
        }
        Ok(body)
    }

    pub async fn poll_until_done(
        &self,
        id: &str,
        poll_interval: Duration,
        timeout: Duration,
    ) -> Result<Transcript, ApiError> {
        let start = tokio::time::Instant::now();
        loop {
            let t = self.get_transcript(id).await?;
            match t.status.as_str() {
                "completed" => return Ok(t),
                "error" => return Ok(t),
                _ => {}
            }

            if start.elapsed() >= timeout {
                return Err(ApiError::Timeout { timeout_seconds: timeout.as_secs() });
            }

            tokio::time::sleep(poll_interval).await;
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("invalid API key header value")]
    InvalidApiKey,

    #[error("failed to build HTTP client: {0}")]
    HttpClientBuild(reqwest::Error),

    #[error("http error: {0}")]
    Http(reqwest::Error),

    #[error("io error: {0}")]
    Io(std::io::Error),

    #[error("json error: {0}")]
    Json(serde_json::Error),

    #[error("api returned HTTP {status}: {body}")]
    HttpStatus { status: reqwest::StatusCode, body: String },

    #[error("transcription failed: {message}")]
    TranscriptError { message: String },

    #[error("timeout after {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },

    #[error("subtitle format is only valid for srt/vtt")]
    InvalidSubtitleFormat,
}

#[derive(Debug, Deserialize)]
struct UploadResponse {
    upload_url: String,
}

#[derive(Debug, Serialize)]
struct CreateTranscriptRequest {
    audio_url: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    speech_model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    language_detection: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    language_code: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    punctuate: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    format_text: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    disfluencies: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    filter_profanity: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    word_boost: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    custom_spelling: Option<Vec<CustomSpellingRequest>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    speech_threshold: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    speaker_labels: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    multichannel: Option<bool>,
}

impl CreateTranscriptRequest {
    fn from_options(audio_url: &str, options: &TranscribeOptions) -> Self {
        let speech_model = match options.speech_model() {
            SpeechModel::Best => Some("best".to_string()),
            SpeechModel::Nano => Some("nano".to_string()),
        };

        let (language_detection, language_code) = match options.language() {
            Language::AutoDetect => (Some(true), None),
            Language::NoDetect => (Some(false), None),
            Language::Fixed { code } => (Some(false), Some(code.clone())),
        };

        let custom_spelling = if options.custom_spelling().is_empty() {
            None
        } else {
            Some(
                options
                    .custom_spelling()
                    .iter()
                    .map(CustomSpellingRequest::from_domain)
                    .collect(),
            )
        };

        let word_boost = if options.word_boost().is_empty() {
            None
        } else {
            Some(options.word_boost().to_vec())
        };

        Self {
            audio_url: audio_url.to_string(),
            speech_model,
            language_detection,
            language_code,
            punctuate: Some(options.punctuate()),
            format_text: Some(options.format_text()),
            disfluencies: Some(options.disfluencies()),
            filter_profanity: Some(options.filter_profanity()),
            word_boost,
            custom_spelling,
            speech_threshold: options.speech_threshold(),
            speaker_labels: Some(options.speaker_labels()),
            multichannel: Some(options.multichannel()),
        }
    }
}

#[derive(Debug, Serialize)]
struct CustomSpellingRequest {
    from: String,
    to: String,
}

impl CustomSpellingRequest {
    fn from_domain(value: &CustomSpelling) -> Self {
        Self {
            from: value.from.clone(),
            to: value.to.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Transcript {
    pub id: String,
    pub status: String,

    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub error: Option<String>,

    #[serde(default)]
    pub utterances: Option<Vec<Utterance>>,
}

#[derive(Debug, Deserialize)]
pub struct Utterance {
    #[serde(default)]
    pub speaker: Option<SpeakerLabel>,

    #[serde(default)]
    pub text: Option<String>,

    #[serde(default)]
    pub start: Option<u64>,

    #[serde(default)]
    pub end: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SpeakerLabel {
    Number(u32),
    Label(String),
}

async fn parse_json_response<T: for<'de> Deserialize<'de>>(resp: reqwest::Response) -> Result<T, ApiError> {
    let status = resp.status();
    let text = resp.text().await.map_err(ApiError::Http)?;
    if !status.is_success() {
        return Err(ApiError::HttpStatus { status, body: text });
    }
    serde_json::from_str(&text).map_err(ApiError::Json)
}
