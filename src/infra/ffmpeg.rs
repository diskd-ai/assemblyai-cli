use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempPath;

use crate::infra::InfraError;

pub struct ExtractedAudio {
    pub path: TempPath,
}

pub fn extract_audio_to_mp3(input_video: &Path) -> Result<ExtractedAudio, InfraError> {
    let temp = tempfile::Builder::new()
        .prefix("assemblyai-cli-")
        .suffix(".mp3")
        .tempfile()?
        .into_temp_path();

    let output_path: PathBuf = temp.to_path_buf();

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-i")
        .arg(input_video)
        .arg("-vn")
        .arg("-ac")
        .arg("1")
        .arg("-codec:a")
        .arg("libmp3lame")
        .arg("-q:a")
        .arg("2")
        .arg(&output_path);

    let output = cmd.output().map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            InfraError::FfmpegNotFound
        } else {
            InfraError::Io(err)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(InfraError::FfmpegFailed { message: stderr });
    }

    Ok(ExtractedAudio { path: temp })
}
