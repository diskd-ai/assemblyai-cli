---
name: assemblyai-cli
description: Install, configure, and troubleshoot `assemblyai-cli` (Rust) to transcribe audio/video into text/srt/vtt using AssemblyAI, including speaker diarization. Use in any agent/tool that supports Agent Skills (Codex, Claude Code, etc.), especially for turning local media files or YouTube links (via `yt-dlp`) into transcripts.
---

# assemblyai-cli

## Install

### GitHub Releases (one-command)

Installs `assemblyai-cli` and attempts to install `ffmpeg` (required for video inputs).

Note: the installer does NOT run `assemblyai-cli init`.

macOS/Linux:
```sh
/bin/bash -c "$(curl -fsSL -H 'Cache-Control: no-cache' https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.sh)"
```

Windows (PowerShell):
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "iwr -useb -Headers @{ 'Cache-Control'='no-cache' } https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.ps1 | iex"
```

### Ensure ffmpeg (video inputs)

macOS (Homebrew):
```sh
command -v ffmpeg >/dev/null 2>&1 || brew install ffmpeg
```

Linux (Debian/Ubuntu):
```sh
command -v ffmpeg >/dev/null 2>&1 || (sudo apt-get update -y && sudo apt-get install -y ffmpeg)
```

Windows (winget):
```powershell
if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) { winget install --id Gyan.FFmpeg -e --accept-package-agreements --accept-source-agreements }
```

### Homebrew (recommended)

Homebrew installs `ffmpeg` automatically (formula dependency).

```sh
brew tap diskd-ai/assemblyai-cli https://github.com/diskd-ai/assemblyai-cli
brew install diskd-ai/assemblyai-cli/assemblyai-cli
```

Upgrade:
```sh
brew update
brew upgrade diskd-ai/assemblyai-cli/assemblyai-cli
```

### GitHub (build from source)

```sh
git clone https://github.com/diskd-ai/assemblyai-cli.git
cd assemblyai-cli
cargo install --path .
```

## Configure

Initialize config interactively (writes/updates `~/.assemblyai-cli/config.json`):
```sh
assemblyai-cli init
```

If `apiKey` already exists, it asks before overwriting. Skip the prompt:
```sh
assemblyai-cli init --yes
```

Alternative: set env var (no config file required):
```sh
export ASSEMBLYAI_API_KEY="..."
```

## Use

Transcribe local audio/video file:
```sh
assemblyai-cli transcribe ./file.mp3
assemblyai-cli transcribe ./file.mp4 --format srt --output ./file.srt
```

Transcribe a direct media URL:
```sh
assemblyai-cli transcribe https://example.com/audio.wav --format vtt
```

Speaker diarization:
```sh
assemblyai-cli transcribe ./file.mp3 --speaker-labels
```

## Video site links (YouTube, etc.) via yt-dlp

`assemblyai-cli transcribe` accepts HTTP(S) URLs, but YouTube/Vimeo/etc. links are usually page URLs, not direct media files. If the user passes a video-site link, download the audio first with `yt-dlp` (https://github.com/yt-dlp/yt-dlp), then transcribe the local file.

If `yt-dlp` is not installed:
- macOS: `brew install yt-dlp`
- Linux: `python3 -m pip install -U yt-dlp`
- Windows: `winget install yt-dlp.yt-dlp`

macOS/Linux (download audio to a known mp3 path, then transcribe):
```sh
yt-dlp -x --audio-format mp3 -o "/tmp/assemblyai-cli-input.%(ext)s" "https://www.youtube.com/watch?v=VIDEO_ID"
assemblyai-cli transcribe "/tmp/assemblyai-cli-input.mp3" --format text --output transcript.txt
```

Windows (PowerShell):
```powershell
yt-dlp -x --audio-format mp3 -o "$env:TEMP\\assemblyai-cli-input.%(ext)s" "https://www.youtube.com/watch?v=VIDEO_ID"
assemblyai-cli transcribe "$env:TEMP\\assemblyai-cli-input.mp3" --format text --output transcript.txt
```

Notes:
- `yt-dlp` often requires `ffmpeg` to extract/convert audio.
- Prefer downloading audio-only (smaller, faster, more reliable) over full video.

## Troubleshoot

- **Missing API key (exit code 3)**: ask the user to run `assemblyai-cli init` (or set `ASSEMBLYAI_API_KEY`) and retry the same command.
- **401 Unauthorized**: API key is invalid; re-run `assemblyai-cli init --yes` or set `ASSEMBLYAI_API_KEY` to a valid key.
- **Video input fails**: ensure `ffmpeg` is installed and on `PATH`.
- **Config not loaded**: verify `~/.assemblyai-cli/config.json` is valid JSON and has `camelCase` keys.
