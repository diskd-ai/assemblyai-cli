use std::path::PathBuf;

use serde::Deserialize;

use crate::domain::{CustomSpelling, SpeechModel, TranscriptFormat};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    #[serde(default)]
    pub api_key: Option<String>,

    #[serde(default)]
    pub base_url: Option<String>,

    #[serde(default)]
    pub format: Option<TranscriptFormat>,

    #[serde(default)]
    pub output: Option<PathBuf>,

    #[serde(default)]
    pub speech_model: Option<SpeechModel>,

    #[serde(default)]
    pub language_detection: Option<bool>,

    #[serde(default)]
    pub language: Option<String>,

    #[serde(default)]
    pub punctuate: Option<bool>,

    #[serde(default)]
    pub format_text: Option<bool>,

    #[serde(default)]
    pub disfluencies: Option<bool>,

    #[serde(default)]
    pub filter_profanity: Option<bool>,

    #[serde(default)]
    pub speaker_labels: Option<bool>,

    #[serde(default)]
    pub multichannel: Option<bool>,

    #[serde(default)]
    pub speech_threshold: Option<f64>,

    #[serde(default)]
    pub chars_per_caption: Option<u32>,

    #[serde(default)]
    pub word_boost: Option<Vec<String>>,

    #[serde(default)]
    pub custom_spelling: Option<Vec<CustomSpelling>>,

    #[serde(default)]
    pub poll_interval_seconds: Option<u64>,

    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_all_fields() {
        let json = r#"
        {
          "apiKey": "abc",
          "baseUrl": "https://api.assemblyai.com",
          "format": "vtt",
          "output": "out.vtt",
          "speechModel": "nano",
          "languageDetection": false,
          "language": "ru",
          "punctuate": true,
          "formatText": true,
          "disfluencies": false,
          "filterProfanity": false,
          "speakerLabels": true,
          "multichannel": false,
          "speechThreshold": 0.2,
          "charsPerCaption": 256,
          "wordBoost": ["one", "two"],
          "customSpelling": [{"from":"a","to":"b"}],
          "pollIntervalSeconds": 2,
          "timeoutSeconds": 900
        }"#;

        let parsed: ConfigFile = serde_json::from_str(json).expect("parse config");
        assert_eq!(parsed.api_key.as_deref(), Some("abc"));
        assert_eq!(parsed.base_url.as_deref(), Some("https://api.assemblyai.com"));
        assert_eq!(parsed.format, Some(TranscriptFormat::Vtt));
        assert_eq!(
            parsed.output.as_ref().and_then(|p| p.to_str()),
            Some("out.vtt")
        );
        assert_eq!(parsed.speech_model, Some(SpeechModel::Nano));
        assert_eq!(parsed.language_detection, Some(false));
        assert_eq!(parsed.language.as_deref(), Some("ru"));
        assert_eq!(parsed.punctuate, Some(true));
        assert_eq!(parsed.format_text, Some(true));
        assert_eq!(parsed.disfluencies, Some(false));
        assert_eq!(parsed.filter_profanity, Some(false));
        assert_eq!(parsed.speaker_labels, Some(true));
        assert_eq!(parsed.multichannel, Some(false));
        assert_eq!(parsed.speech_threshold, Some(0.2));
        assert_eq!(parsed.chars_per_caption, Some(256));
        assert_eq!(
            parsed.word_boost.as_deref(),
            Some(&["one".to_string(), "two".to_string()][..])
        );
        assert_eq!(
            parsed.custom_spelling.as_deref(),
            Some(&[CustomSpelling {
                from: "a".to_string(),
                to: "b".to_string()
            }][..])
        );
        assert_eq!(parsed.poll_interval_seconds, Some(2));
        assert_eq!(parsed.timeout_seconds, Some(900));
    }
}
