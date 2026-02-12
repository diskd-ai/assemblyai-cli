# AssemblyAI CLI Skill

> **Install:** `npx skills add diskd-ai/assemblyai-cli --skill assemblyai-cli --global --yes` | [skills.sh](https://skills.sh)

Agent Skill for installing and using `assemblyai-cli`, a Rust CLI that transcribes audio/video into `text`, `srt`, or `vtt` using the AssemblyAI API (optional speaker diarization).

Supported agents/tools (examples): Codex, Claude (Claude Code), Pi, clawd.bot, openclaw.

---

## Scope & Purpose

This skill helps an agent/tool:

- Install `assemblyai-cli` (Homebrew or GitHub Releases)
- Ensure `ffmpeg` is available for video inputs
- Initialize `~/.assemblyai-cli/config.json` using `assemblyai-cli init` (only when needed)
- Run `assemblyai-cli transcribe` with the right flags for:
  - Plain text transcripts
  - Subtitles (SRT/VTT)
  - Speaker diarization (`--speaker-labels`)
- Turn YouTube or other streaming links into transcripts by downloading audio with `yt-dlp` first

---

## When to Use This Skill

Triggers:

- The user asks to transcribe audio/video using AssemblyAI
- The user needs SRT/VTT subtitles or diarized output
- The user provides a YouTube link and wants a transcript (use `yt-dlp` to download audio first)
- The user is missing API key/config and needs a fix

---

## Quick Start

1) Install `assemblyai-cli`:

- Homebrew:
  - `brew tap diskd-ai/assemblyai-cli https://github.com/diskd-ai/assemblyai-cli`
  - `brew install diskd-ai/assemblyai-cli/assemblyai-cli`
- GitHub Releases (macOS/Linux):
  - `/bin/bash -c "$(curl -fsSL -H 'Cache-Control: no-cache' https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.sh)"`
- GitHub Releases (Windows PowerShell):
  - `powershell -NoProfile -ExecutionPolicy Bypass -Command "iwr -useb -Headers @{ 'Cache-Control'='no-cache' } https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.ps1 | iex"`

2) Transcribe:

- Local file:
  - `assemblyai-cli transcribe ./file.mp3 --output transcript.txt`
- Subtitles:
  - `assemblyai-cli transcribe ./file.mp4 --format srt --output transcript.srt`

3) If `assemblyai-cli` exits with “missing AssemblyAI API key”, ask the user to run:

- `assemblyai-cli init`

Then retry the same transcription command.

---

## YouTube Example (via yt-dlp)

`assemblyai-cli` accepts HTTP(S) URLs, but YouTube (and similar video sites) usually provide page URLs, not direct media files. Download audio first with `yt-dlp`, then transcribe the local file.

Install `yt-dlp` if needed (https://github.com/yt-dlp/yt-dlp):

```sh
# macOS
brew install yt-dlp

# Linux
python3 -m pip install -U yt-dlp

# Windows (PowerShell)
winget install yt-dlp.yt-dlp
```

```sh
yt-dlp -x --audio-format mp3 -o "/tmp/assemblyai-cli-input.%(ext)s" "https://www.youtube.com/watch?v=VIDEO_ID"
assemblyai-cli transcribe "/tmp/assemblyai-cli-input.mp3" --output transcript.txt
```

---

## Resources

- Skill entry point: [SKILL.md](SKILL.md)
- Repository: https://github.com/diskd-ai/assemblyai-cli
