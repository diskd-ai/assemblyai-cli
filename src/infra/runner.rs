use std::path::Path;

use crate::app::TranscribePlan;
use crate::domain::{Output, TranscriptFormat, TranscribeOptions};
use crate::infra::assemblyai::{ApiError, AssemblyAiClient, SpeakerLabel, Transcript};
use crate::infra::{ffmpeg, InfraError};

pub async fn run_transcribe(plan: TranscribePlan, client: AssemblyAiClient, options: &TranscribeOptions) -> Result<(), RunnerError> {
    match plan {
        TranscribePlan::Url { url } => {
            let transcript = transcribe_audio_url(&client, url.as_str(), options).await?;
            write_output(transcript, options)?;
            Ok(())
        }
        TranscribePlan::LocalAudio { path } => {
            let transcript = transcribe_local_file(&client, &path, options).await?;
            write_output(transcript, options)?;
            Ok(())
        }
        TranscribePlan::LocalVideoExtract { path } => {
            if !path.exists() {
                return Err(RunnerError::Infra(InfraError::InputNotFound { path }));
            }
            let extracted = ffmpeg::extract_audio_to_mp3(&path)?;
            let transcript = transcribe_local_file(&client, extracted.path.as_ref(), options).await?;
            write_output(transcript, options)?;
            Ok(())
        }
    }
}

fn write_output(content: String, options: &TranscribeOptions) -> Result<(), InfraError> {
    match options.output() {
        Output::Stdout => {
            print!("{content}");
            Ok(())
        }
        Output::FilePath(path) => {
            std::fs::write(path, content)?;
            eprintln!("wrote transcript to {}", path.display());
            Ok(())
        }
    }
}

async fn transcribe_local_file(client: &AssemblyAiClient, path: &Path, options: &TranscribeOptions) -> Result<String, RunnerError> {
    if !path.exists() {
        return Err(RunnerError::Infra(InfraError::InputNotFound {
            path: path.to_path_buf(),
        }));
    }

    eprintln!("uploading: {path:?}");
    let upload_url = client.upload_file(path).await?;
    transcribe_audio_url(client, &upload_url, options).await
}

async fn transcribe_audio_url(client: &AssemblyAiClient, audio_url: &str, options: &TranscribeOptions) -> Result<String, RunnerError> {
    eprintln!("starting transcription");
    let created = client.create_transcript(audio_url, options).await?;
    let done = client
        .poll_until_done(&created.id, options.poll_interval(), options.timeout())
        .await?;

    if done.status.as_str() == "error" {
        return Err(RunnerError::Api(ApiError::TranscriptError {
            message: done.error.unwrap_or_else(|| "unknown transcription error".to_string()),
        }));
    }

    match options.format() {
        TranscriptFormat::Text => Ok(format_text_output(&done, options)),
        TranscriptFormat::Srt => match format_diarized_subtitles(&done, options) {
            Some(value) => Ok(value),
            None => Ok(client
                .get_subtitles(&done.id, options.format(), options.chars_per_caption())
                .await?),
        },
        TranscriptFormat::Vtt => match format_diarized_subtitles(&done, options) {
            Some(value) => Ok(value),
            None => Ok(client
                .get_subtitles(&done.id, options.format(), options.chars_per_caption())
                .await?),
        },
    }
}

fn format_text_output(done: &Transcript, options: &TranscribeOptions) -> String {
    if options.speaker_labels() {
        if let Some(value) = diarized_utterances(done)
            .as_ref()
            .map(|u| crate::domain::subtitles::format_diarized_text(u))
        {
            if !value.trim().is_empty() {
                return value;
            }
        }
    }

    done.text.clone().unwrap_or_default()
}

fn speaker_to_string(value: &SpeakerLabel) -> String {
    match value {
        SpeakerLabel::Number(n) => n.to_string(),
        SpeakerLabel::Label(s) => s.clone(),
    }
}

fn format_diarized_subtitles(done: &Transcript, options: &TranscribeOptions) -> Option<String> {
    if !options.speaker_labels() {
        return None;
    }

    let utterances = diarized_utterances(done)?;
    if utterances.is_empty() {
        return None;
    }

    let result = match options.format() {
        TranscriptFormat::Srt => crate::domain::subtitles::format_diarized_srt(&utterances, options.chars_per_caption()),
        TranscriptFormat::Vtt => crate::domain::subtitles::format_diarized_vtt(&utterances, options.chars_per_caption()),
        TranscriptFormat::Text => return None,
    };

    if result.trim().is_empty() {
        None
    } else {
        Some(result)
    }
}

fn diarized_utterances(done: &Transcript) -> Option<Vec<crate::domain::subtitles::DiarizedUtterance>> {
    let utterances = done.utterances.as_ref()?;
    let mut out: Vec<crate::domain::subtitles::DiarizedUtterance> = Vec::new();

    for utterance in utterances {
        let speaker = utterance
            .speaker
            .as_ref()
            .map(speaker_to_string)
            .unwrap_or_else(|| "Unknown".to_string());
        let text = utterance.text.clone().unwrap_or_default();
        let Some(start_ms) = utterance.start else { continue };
        let Some(end_ms) = utterance.end else { continue };

        if let Some(value) = crate::domain::subtitles::DiarizedUtterance::new(start_ms, end_ms, speaker, text) {
            out.push(value);
        }
    }

    Some(out)
}

#[derive(thiserror::Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]
    Infra(#[from] InfraError),

    #[error(transparent)]
    Api(#[from] ApiError),
}
