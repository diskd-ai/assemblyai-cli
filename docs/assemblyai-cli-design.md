AssemblyAI CLI (Rust) Design Doc
================================

Context and motivation
----------------------
`assembly_ai_reader.py` provides an AssemblyAI-backed transcription reader in Python, including optional video->audio extraction and multiple transcript output formats. This repo needs a standalone Rust CLI that offers the same core capability (transcribe audio/video files) without requiring Python or LlamaIndex, so it can be used in shell pipelines and automation.

Goals:
- Provide a CLI-only tool `assemblyai-cli` that transcribes a single audio/video input (local path or HTTP(S) URL) via AssemblyAI.
- Resolve the API key from `~/.assemblyai-cli/config.json` (or legacy `~/.assemblyai-cli`) with fallback to environment variables (no interactive prompts).
- Support transcript output formats: `text`, `srt`, `vtt` (aligned with `TranscriptFormat` in `assembly_ai_reader.py`).
- Support a focused set of transcription options aligned with the Python reader (model, language detection/language, punctuation, formatting, disfluencies, profanity filtering, speaker labels, multichannel, word boost, custom spelling, chars-per-caption).
- Provide deterministic exit codes and clear, actionable error messages on stderr.
- Keep a strict purity boundary: domain/application layers are pure and return typed `Result` values; all I/O (filesystem, ffmpeg, network) lives in infrastructure/entrypoint code.

Non-goals for first implementation (v1)
---------------------------------------
- Batch mode (multiple inputs per invocation) and concurrency.
- Streaming transcripts / realtime transcription.
- Advanced AssemblyAI features not present in `assembly_ai_reader.py` (summarization, topic detection, etc.).
- Persisting transcript jobs for resume/retry across runs.
- Interactive authentication flows.
- Cross-platform bundled media conversion (v1 relies on an existing `ffmpeg` binary when video extraction is required).

Implementation considerations
-----------------------------
- Rust target: stable Rust (edition 2024) with a single binary crate.
- CLI parsing: `clap` (derive) for consistent help/validation.
- HTTP client: `reqwest` with TLS; JSON via `serde`.
- Tempfiles: `tempfile` for safe cleanup of extracted audio.
- Purity boundary:
  - `domain/`: pure types + validation.
  - `app/`: pure orchestration/state machine that decides the next command given the current state and the last result.
  - `infra/`: effectful executors for filesystem/ffmpeg/HTTP and an async runtime that drives the app state machine.
  - `main.rs`: composition root; reads env/args, wires infra, maps errors to exit codes.
- Error handling: use explicit error enums (`DomainError`, `InfraError`, `ApiError`) and return `Result` values; avoid panics for expected failures.
- Video handling: follow the Python reader’s intent (extract mono audio from common video containers) to reduce variability. Extraction uses `ffmpeg` via `std::process::Command` and produces a temporary `.mp3` for upload.

High-level behavior
-------------------
Given a single input (file path or URL), the CLI:
1. Parses CLI arguments into a `domain::TranscribeOptions`.
2. Loads optional configuration from `~/.assemblyai-cli/config.json` (or legacy `~/.assemblyai-cli`) and resolves the API key:
   - `apiKey` in config
   - `ASSEMBLYAI_API_KEY`
   - `ASSEMBLY_AI_KEY` (base64-encoded; decoded automatically if it looks like base64)
   If the key is still missing, exits with a configuration error.
3. Determines input kind:
   - If argument starts with `http://` or `https://`, treat as URL input.
   - Otherwise treat as local filesystem path.
4. For local files:
   - Validates the file exists and is readable.
   - If the extension is a known video container, extracts audio to a temporary `.mp3` using `ffmpeg`.
5. Uploads the audio (or passes the URL) to AssemblyAI, creates a transcript job with the requested config, and polls until the job is `completed` or `error`.
6. Fetches the transcript in the requested format:
   - `text`: prints the plain transcript by default, but when `--speaker-labels` is enabled it prefers utterance-based output (`Speaker X: ...`) when available.
   - `srt`/`vtt`: fetches subtitle exports with `chars_per_caption`, but when `--speaker-labels` is enabled it prefers utterance-based subtitles (`Speaker X: ...`) when available.
7. Writes the result:
   - If `--output` is provided, writes to that file path.
   - Otherwise prints to stdout (UTF-8), writing progress/status to stderr.
8. Cleans up any temporary extracted audio and exits `0` on success.

CLI interface
-------------
Command:
- `assemblyai-cli transcribe <INPUT>`
- `assemblyai-cli init`

`<INPUT>`:
- Local file path (audio or video), or
- HTTP(S) URL.

Flags (v1):
- `--format <text|srt|vtt>` (default: `text`)
- `--output <PATH>` (default: stdout)
- `--speech-model <best|nano>` (default: `best`)
- `--language-detection` / `--no-language-detection` (default: enabled)
- `--language <CODE>` (only valid when `--no-language-detection`)
- `--punctuate` / `--no-punctuate` (default: enabled)
- `--format-text` / `--no-format-text` (default: enabled)
- `--disfluencies` (default: disabled)
- `--filter-profanity` (default: disabled)
- `--speaker-labels` (default: disabled; when used with `--format text`, outputs `Speaker X: ...` lines when available)
- `--multichannel` / `--no-multichannel` (default: enabled, matches Python default)
- `--speech-threshold <0.0..1.0>` (optional)
- `--chars-per-caption <N>` (default: 128; only used for `srt`/`vtt`)
- `--word-boost <PHRASE>` (repeatable)
- `--custom-spelling <FROM=TO>` (repeatable; maps to Python `custom_spelling`)
- `--poll-interval-seconds <N>` (default: 3)
- `--timeout-seconds <N>` (default: 3600)

Environment variables:
- `ASSEMBLYAI_API_KEY` (required when config `apiKey` is not provided)
- `ASSEMBLYAI_BASE_URL` (optional; default: `https://api.assemblyai.com`; the Python reader uses `https://api.eu.assemblyai.com`)
- `ASSEMBLY_AI_KEY` (optional; base64-encoded API key)

Config file:
- Path: `~/.assemblyai-cli/config.json` (preferred) or `~/.assemblyai-cli` (legacy).
- Format: JSON, `camelCase` keys.
- Precedence: CLI flags override config values.
- Initialization: `assemblyai-cli init` prompts for an API key and writes/updates the config file. If `apiKey` already exists, it asks before overwriting unless `--yes` is provided.

Example usage:
```
export ASSEMBLYAI_API_KEY="..."
assemblyai-cli transcribe ./audio.m4a
assemblyai-cli transcribe ./video.mp4 --format srt --output ./video.srt
assemblyai-cli transcribe https://example.com/audio.wav --format vtt
```

Input validation and media handling
-----------------------------------
Allowlisted extensions (v1):
- Audio: `.mp3`, `.wav`, `.flac`, `.m4a`, `.ogg`
- Video: `.mp4`, `.avi`, `.mov`, `.mkv`, `.webm`

Rules:
- If `<INPUT>` is a local path and the extension is not allowlisted, return a `DomainError::UnsupportedExtension`.
- If `<INPUT>` is a URL, skip extension validation (AssemblyAI may still reject it; that becomes an API error).
- Video extraction:
  - Require `ffmpeg` to be present on `PATH`.
  - Extract mono audio (single channel) to a temporary `.mp3`.
  - Ensure temporary files are removed even on failure paths.

AssemblyAI API interaction model
--------------------------------
The CLI uses AssemblyAI’s asynchronous transcription workflow:
- Upload local files to the upload endpoint (streaming file bytes to avoid loading entire files into memory).
- Create a transcript job with a JSON config derived from `domain::TranscribeOptions`.
- Poll transcript status until:
  - `completed`: fetch transcript output.
  - `error`: surface the error message and exit non-zero.
- For `srt`/`vtt`, fetch subtitle exports with `chars_per_caption`.

Data model (domain)
-------------------
Key domain types are expressed as ADTs to prevent invalid combinations:
- `SpeechModel = Best | Nano`
- `TranscriptFormat = Text | Srt | Vtt`
- `Input = LocalPath(PathBuf) | Url(String)`
- `Language = AutoDetect | Fixed { code: String }`
- `Output = Stdout | FilePath(PathBuf)`
- `TranscribeOptions` (product type) contains only validated combinations, e.g. it is impossible to construct `Language::Fixed` while `language_detection` is enabled.

Error handling and UX
---------------------
User-facing behavior:
- All errors print a single-line summary to stderr prefixed with `error:` and a short hint when actionable.
- Successful transcripts print only transcript content to stdout (unless `--output` is used).

Error categories and exit codes (v1):
- `2`: CLI usage / validation error (bad flag value, unsupported extension, invalid `speech-threshold`)
- `3`: Missing/invalid configuration (missing `ASSEMBLYAI_API_KEY`)
- `4`: External dependency missing (e.g., `ffmpeg` not found when needed)
- `5`: AssemblyAI API error (HTTP non-2xx, transcript status `error`)
- `1`: Unexpected runtime failure (I/O errors, JSON parse errors, etc., after mapping)

Update cadence / Lifecycle
--------------------------
This is a run-to-completion CLI:
- Polls AssemblyAI at a fixed interval (`--poll-interval-seconds`) until completion or timeout (`--timeout-seconds`).
- No background daemons or persisted state in v1.

Future-proofing
---------------
- Add subcommands without breaking existing flags (`transcribe` remains stable).
- Extend `domain::TranscribeOptions` via new fields with defaults; avoid changing existing flag semantics.
- Add batch mode as a new subcommand (`transcribe-batch`) rather than overloading `transcribe`.
- Support additional outputs (JSON metadata, word-level timestamps) as opt-in flags.
- Support config file loading as an additive feature (e.g., `--config`), while keeping `ASSEMBLYAI_API_KEY` env as the primary key source.

Implementation outline
----------------------
1. Create Rust project structure:
   - `src/main.rs` (composition root)
   - `src/domain/` (types + validation)
   - `src/app/` (pure state machine / orchestrator)
   - `src/infra/` (ffmpeg runner, filesystem, AssemblyAI HTTP client, polling runtime)
2. Implement CLI parsing and mapping to `domain::TranscribeOptions` with validation.
3. Implement media preparation:
   - Detect input kind (URL vs path)
   - For video paths, run `ffmpeg` extraction into `tempfile`
4. Implement AssemblyAI client in `infra`:
   - Upload local files
   - Create transcript job
   - Poll status
   - Fetch output for requested format
5. Implement entrypoint wiring:
   - Read `ASSEMBLYAI_API_KEY` (and optional base URL) from env in `main.rs`
   - Map errors to exit codes/messages
6. Add documentation:
   - `README.md` usage (v1) and installation notes (ffmpeg requirement for video inputs)

Testing approach
----------------
Unit tests (pure):
- `domain` validation: flag combinations, `speech-threshold` bounds, extension allowlist, parsing of `custom-spelling`.
- `app` state machine transitions: given API results/events, the next command is correct.

Integration tests (optional, require network + API key):
- Guarded by checking `ASSEMBLYAI_API_KEY` at runtime; skip if missing.
- Use a small audio fixture in `tests/fixtures/` (or generate a synthetic short WAV) to validate end-to-end transcription.

Manual tests:
- Transcribe an audio file to stdout (`text`).
- Transcribe a video file to `srt` with `--output`.
- Validate behavior when `ASSEMBLYAI_API_KEY` is missing.

Acceptance criteria
-------------------
- Running `assemblyai-cli transcribe <audio-file>` with `ASSEMBLYAI_API_KEY` set prints a non-empty transcript to stdout and exits `0`.
- Running `assemblyai-cli transcribe <video-file>` extracts audio using `ffmpeg`, transcribes successfully, removes temporary files, and exits `0`.
- `--format srt` and `--format vtt` produce subtitle-formatted output and respect `--chars-per-caption`.
- Missing API key (no config `apiKey` and no env key) results in a clear stderr error and exit code `3`.
- Unsupported local file extensions fail fast with exit code `2` and an actionable message.
- AssemblyAI transcription failures surface the provider error message and exit code `5`.
- When `--speaker-labels` is enabled (or `speakerLabels: true` in config), `text` output is diarized (`Speaker X: ...`) when utterances are available, and `srt`/`vtt` prefer diarized subtitles.
- When config is present at `~/.assemblyai-cli/config.json` and contains `apiKey`, the CLI runs without requiring `ASSEMBLYAI_API_KEY` in the environment.
