assemblyai-cli
===============

Rust CLI to transcribe audio/video files via AssemblyAI.

Quickstart
----------
1. Install Rust toolchain
2. Initialize config (recommended):
   - `assemblyai-cli init` (or `cargo run --quiet -- init` when running from source)
3. Or export API key:
   - `export ASSEMBLYAI_API_KEY="..."`
4. Transcribe:
   - `assemblyai-cli transcribe demo/part3.mp3 --output part3.txt` (or `cargo run --quiet -- transcribe demo/part3.mp3 --output part3.txt`)

Commands
--------
- `assemblyai-cli transcribe <INPUT>`
- `assemblyai-cli init`

`<INPUT>`:
- Local file path (audio/video), or
- HTTP(S) URL.

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
  "languageDetection": true,
  "language": "ru",

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

Tests
-----
Acceptance tests use `demo/part3.{mp3,mp4}` and load the API key from `.env`.

- `RUST_TEST_THREADS=1 cargo test`
