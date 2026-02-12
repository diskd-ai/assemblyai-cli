assemblyai-cli
===============

Rust CLI to transcribe audio/video files via AssemblyAI.

This repo also ships an Agent Skill (via [skills.sh](https://skills.sh)) that works with Codex, Claude (Claude Code), Pi, clawd.bot, openclaw, and other Agent Skills-compatible tools.

Install
-------
Quick install from GitHub Releases (macOS/Linux):
```sh
/bin/bash -c "$(curl -fsSL -H 'Cache-Control: no-cache' https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.sh)"
```

Quick install from GitHub Releases (Windows PowerShell):
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "iwr -useb -Headers @{ 'Cache-Control'='no-cache' } https://raw.githubusercontent.com/diskd-ai/assemblyai-cli/main/scripts/install.ps1 | iex"
```

After installing, run `assemblyai-cli init` (recommended) or set `ASSEMBLYAI_API_KEY` to configure authentication.

Homebrew (recommended):
```sh
brew tap diskd-ai/assemblyai-cli https://github.com/diskd-ai/assemblyai-cli
brew install diskd-ai/assemblyai-cli/assemblyai-cli
```

Upgrade:
```sh
brew update
brew upgrade diskd-ai/assemblyai-cli/assemblyai-cli
```

From source:
```sh
git clone https://github.com/diskd-ai/assemblyai-cli.git
cd assemblyai-cli
cargo install --path .
```

Skill (skills.sh)
-----------------
This repo ships a skill in `skill/assemblyai-cli`.

Supported agents/tools (examples): Codex, Claude (Claude Code), Pi, clawd.bot, openclaw.

List available skills in this repo:
```sh
npx skills add diskd-ai/assemblyai-cli --list
```

Install the `assemblyai-cli` skill globally:
```sh
npx skills add diskd-ai/assemblyai-cli --skill assemblyai-cli --global --yes
```

Example invocation (Agent Skills, e.g. Claude Code):
```
/assemblyai-cli install via homebrew, then transcribe ./file.mp3
```

Quickstart
----------
1. Install `assemblyai-cli`
2. Initialize config (recommended):
   - `assemblyai-cli init` (or `cargo run --quiet -- init` when running from source)
3. Or export API key:
   - `export ASSEMBLYAI_API_KEY="..."`
4. Transcribe:
   - `assemblyai-cli transcribe ./file.mp3 --output transcript.txt` (or `cargo run --quiet -- transcribe ./file.mp3 --output transcript.txt`)

Commands
--------
- `assemblyai-cli transcribe <INPUT>`
- `assemblyai-cli init`

`<INPUT>`:
- Local file path (audio/video), or
- HTTP(S) URL.

YouTube and other streaming URLs are usually not direct media URLs. Download audio with `yt-dlp` first, then transcribe the local file.

Example (YouTube -> mp3 -> transcript):
```sh
yt-dlp -x --audio-format mp3 -o "/tmp/assemblyai-cli-input.%(ext)s" "https://www.youtube.com/watch?v=VIDEO_ID"
assemblyai-cli transcribe "/tmp/assemblyai-cli-input.mp3" --output transcript.txt
```

Supported formats:
- `--format text` (default)
- `--format srt`
- `--format vtt`

Speaker diarization:
- `--speaker-labels` (when used with `--format text`, prints `Speaker X: ...`)
- When used with `--format srt|vtt`, it prefers utterance-based subtitles with `Speaker X: ...` when available.

Configuration
-------------
The CLI looks for a JSON config at:
- `~/.assemblyai-cli/config.json` (preferred), or
- `~/.assemblyai-cli` (legacy single-file config).

To create or update the config interactively:
- `assemblyai-cli init` (prompts for API key; if `apiKey` already exists it asks before overwriting; use `--yes` to skip the prompt)

API key resolution order:
1. Config `apiKey`
2. `ASSEMBLYAI_API_KEY`
3. `ASSEMBLY_AI_KEY` (base64 encoded; decoded automatically if it looks like base64)

CLI flags override config values.

Config file schema (JSON, camelCase)
------------------------------------
Example `~/.assemblyai-cli/config.json`:
```json
{
  "apiKey": "YOUR_ASSEMBLYAI_API_KEY",
  "baseUrl": "https://api.assemblyai.com",

  "format": "text",
  "output": "transcript.txt",

  "speechModel": "best",
  "languageDetection": false,
  "language": "fr",

  "punctuate": true,
  "formatText": true,
  "disfluencies": false,
  "filterProfanity": false,

  "speakerLabels": false,
  "multichannel": true,
  "speechThreshold": 0.1,

  "charsPerCaption": 128,
  "wordBoost": ["MyProject"],
  "customSpelling": [{ "from": "MyProject", "to": "MyProject" }],

  "pollIntervalSeconds": 3,
  "timeoutSeconds": 3600
}
```

Notes:
- `output` is optional; when omitted, transcript prints to stdout.
- `customSpelling` is a list of `{ "from": "...", "to": "..." }` objects.

Video inputs
------------
For video files (`.mp4`, `.avi`, `.mov`, `.mkv`, `.webm`), the CLI extracts audio using `ffmpeg` (must be available on `PATH`).

Homebrew troubleshooting
------------------------
If you previously used a debug tap (for example `alexeus/assemblyai-cli`), Homebrew may warn about missing `origin` remotes or “Formulae found in multiple taps”. Fix by removing the old tap and reinstalling from the `diskd-ai` tap:

```sh
brew untap alexeus/assemblyai-cli || true
brew tap diskd-ai/assemblyai-cli https://github.com/diskd-ai/assemblyai-cli
brew install diskd-ai/assemblyai-cli/assemblyai-cli
```

Tests
-----
Tests cover CLI/config behavior without requiring bundled media files.

Optional end-to-end transcription testing requires a valid API key and your own media file(s).

- `RUST_TEST_THREADS=1 cargo test`
