---
name: assemblyai-cli
description: Install, configure, and troubleshoot the `assemblyai-cli` Rust CLI for transcribing audio/video with AssemblyAI. Use for Homebrew installation (tap `diskd-ai/assemblyai-cli`), building from GitHub (`diskd-ai/assemblyai-cli`), initializing `~/.assemblyai-cli/config.json` via `assemblyai-cli init`, and running `assemblyai-cli transcribe` (text/srt/vtt, optional diarization).
---

# assemblyai-cli

## Install

### Homebrew (recommended)

```sh
brew tap diskd-ai/assemblyai-cli
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

Transcribe URL:
```sh
assemblyai-cli transcribe https://example.com/audio.wav --format vtt
```

Speaker diarization:
```sh
assemblyai-cli transcribe ./file.mp3 --speaker-labels
```

## Troubleshoot

- **401 Unauthorized**: API key is missing/invalid; re-run `assemblyai-cli init --yes` or set `ASSEMBLYAI_API_KEY`.
- **Video input fails**: ensure `ffmpeg` is installed and on `PATH`.
- **Config not loaded**: verify `~/.assemblyai-cli/config.json` is valid JSON and has `camelCase` keys.
