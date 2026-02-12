pub mod assemblyai;
pub mod ffmpeg;
pub mod runner;

#[derive(thiserror::Error, Debug)]
pub enum InfraError {
    #[error("input file not found: {path:?}")]
    InputNotFound { path: std::path::PathBuf },

    #[error("ffmpeg not found on PATH")]
    FfmpegNotFound,

    #[error("ffmpeg failed: {message}")]
    FfmpegFailed { message: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl InfraError {
    pub fn exit_code(&self) -> u8 {
        match self {
            InfraError::InputNotFound { .. } => 2,
            InfraError::FfmpegNotFound | InfraError::FfmpegFailed { .. } => 4,
            _ => 1,
        }
    }
}
