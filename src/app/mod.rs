use crate::domain::{Input, TranscribeOptions};

#[derive(Debug, Clone)]
pub enum TranscribePlan {
    Url { url: url::Url },
    LocalAudio { path: std::path::PathBuf },
    LocalVideoExtract { path: std::path::PathBuf },
}

pub fn build_plan(options: &TranscribeOptions) -> Result<TranscribePlan, crate::domain::DomainError> {
    match options.input() {
        Input::Url(url) => Ok(TranscribePlan::Url { url: url.clone() }),
        Input::LocalPath(path) => match crate::domain::classify_local_media(path) {
            crate::domain::MediaKind::Audio => Ok(TranscribePlan::LocalAudio { path: path.clone() }),
            crate::domain::MediaKind::Video => Ok(TranscribePlan::LocalVideoExtract { path: path.clone() }),
            crate::domain::MediaKind::Unknown => Err(crate::domain::DomainError::UnsupportedExtension {
                path: path.clone(),
            }),
        },
    }
}
