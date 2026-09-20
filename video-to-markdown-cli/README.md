# Video-to-Markdown CLI

Internal, maintainer-facing tool that turns a local video into a reviewed
Markdown transcript under `video-from-text/`. It runs four stages in order —
audio extraction, MiMo V2.5 ASR transcription, constrained cleanup, and
atomic Markdown output. See `epics/07-video-to-markdown-cli.md` for the full
design and rationale; this document is the practical walkthrough for someone
about to run it.

This is a content-production tool for turning the project's own videos into
text that can support the website's learning content. It is not a
visitor-facing upload or transcription feature.

## Setup

1. **Install `ffmpeg` and make sure it's on `PATH`.** The audio stage invokes
   the `ffmpeg` binary directly (no shell) to extract 128 kbps MP3 audio
   (`libmp3lame`) from the source video. Verify it's available before running
   the CLI:

   ```sh
   ffmpeg -version
   ```

   If that fails, install `ffmpeg` with your platform's package manager (for
   example `brew install ffmpeg` on macOS) and re-check.

2. **Set the `XIAOMI_API_KEY` environment variable.** Never commit this key
   or pass it as a CLI argument — the tool only reads it from the
   environment, and no module in this crate logs, prints, or otherwise
   echoes it.

   ```sh
   export XIAOMI_API_KEY="..."
   ```

   Alternatively, create a `.env` file at the **repository root** (not inside
   `video-to-markdown-cli/`):

   ```sh
   # .env (repository root)
   XIAOMI_API_KEY=your-key-here
   ```

   The CLI loads it automatically on startup, before reading any environment
   variables. A variable already exported in your shell always takes
   precedence over the same name in `.env`. `.env` is listed in the repo's
   `.gitignore` — never commit it.

| Variable | Description | Required |
|----------|--------------|----------|
| `XIAOMI_API_KEY` | Xiaomi API key used for both MiMo V2.5 ASR (transcription) and the MiMo V2.5 chat cleanup stage. Read once per run and reused for both calls. May be set in the shell environment or in a repository-root `.env` file. | Yes |

## A normal run

```sh
cargo run -p video-to-markdown-cli -- <path-to-video> [--overwrite]
```

If you've built a release binary (`cargo build --release -p video-to-markdown-cli`),
you can also run it directly as `video-to-markdown`:

```sh
./target/release/video-to-markdown <path-to-video> [--overwrite]
```

- `<path-to-video>`: path to a local video file to transcribe.
- `--overwrite`: required to replace an existing transcript already present
  under `video-from-text/`; without it, a colliding run is refused.

What happens, in order:

1. **Audio extraction.** The video is validated as a readable file, `ffmpeg`
   is confirmed to be on `PATH`, and a 128 kbps MP3 file is extracted
   (`ffmpeg -vn -c:a libmp3lame -b:a 128k`) into a per-run temporary
   workspace (never overwriting the source video). MP3 was chosen over
   uncompressed WAV because it keeps longer recordings well under the
   provider's 10 MB Base64 size limit below.
2. **ASR transcription.** The extracted audio is Base64-encoded, checked
   against the provider's 10 MB request-size limit, and sent to MiMo V2.5
   ASR via `llm-sdk`'s Xiaomi client to produce a raw transcript.
3. **Cleanup.** The raw transcript is sent to a separate MiMo V2.5 chat call
   with a dedicated system prompt (`video-to-markdown-cli/prompts/cleanup_system_prompt.txt`)
   that corrects only spelling, punctuation, capitalization, and grammar, and
   adds paragraph breaks — nothing else.
4. **Markdown output.** The cleaned transcript is written atomically (via a
   temporary file and rename) as UTF-8 Markdown under `video-from-text/`.

Progress for each stage is printed to stderr; only the final Markdown path is
printed to stdout, so the command is composable in scripts:

```sh
md_path=$(cargo run -p video-to-markdown-cli -- talk.mp4)
echo "wrote $md_path"
```

### Output location

The transcript is written to `video-from-text/<video-stem>.md`, relative to
the current working directory, using a filename derived from the source
video's filename stem (for example, `my-talk.mp4` produces
`video-from-text/my-talk.md`). If your video's filename contains spaces,
punctuation, or other characters outside `A-Z a-z 0-9 - _`, those characters
are replaced with `-` (runs collapsed, leading/trailing dashes trimmed) to
produce a safe, predictable filename — see the sanitization rules documented
at the top of `src/output.rs` for the exact behavior (including the
fallback name `transcript` for a stem that sanitizes away to nothing).

### Overwrite behavior

Re-running the command against a video whose derived output already exists
under `video-from-text/` fails by default rather than replacing a reviewed
transcript. Pass `--overwrite` to explicitly allow replacing it.

## Common failures

| Failure | What it means | What to do |
|---------|----------------|-------------|
| `video file not found` / `cannot read video file ...` | The given path doesn't exist, isn't a regular file, or isn't readable. | Check the path and file permissions. |
| `ffmpeg was not found on PATH` | `ffmpeg` isn't installed or isn't on `PATH`. | Install `ffmpeg` (see Setup) and confirm `ffmpeg -version` works. |
| `ffmpeg failed (...)` / `ffmpeg did not produce an audio file` | The `ffmpeg` process exited non-zero or produced no output — usually an unsupported or corrupt video. | Try re-encoding the source video, or confirm it plays correctly elsewhere. |
| `XIAOMI_API_KEY is not set` | The environment variable is missing or empty in this shell. | `export XIAOMI_API_KEY=...` before running. |
| `extracted audio Base64-encodes to ... bytes, which exceeds the provider's ... (10 MB) limit` | The extracted audio is too large for a single ASR request. This tool deliberately does not segment audio into multiple requests (splitting audio would split sentences/paragraphs across independent ASR calls and complicate reassembly). | Trim the source video to a shorter segment, or re-export it at a lower bitrate, then re-run against the smaller input. |
| `Xiaomi ASR authentication failed` / `cleanup authentication failed` | The API key is invalid, revoked, or otherwise rejected. | Check that `XIAOMI_API_KEY` holds a current, valid key. |
| `Xiaomi ASR rate limit exceeded` / `cleanup rate limit exceeded` | The provider throttled this request. | Wait and retry. |
| `network error while calling Xiaomi ASR` / `... the cleanup agent` | A transport-level failure (DNS, TLS, connection reset, timeout). | Check network connectivity and retry. |
| `Xiaomi ASR returned an error (status ...)` / `cleanup agent returned an error (status ...)` | The provider rejected the request or had a server-side problem not covered by a more specific case. | Read the included provider message; retry if transient. |
| `Xiaomi ASR returned an empty transcript` | The audio was silent, unclear, or in an unsupported language. | Check the source audio and language. |
| `cleanup agent returned an empty result` | The cleanup call returned a blank response. | Retry; if it persists, treat as a provider-side issue. |
| `a transcript already exists at video-from-text/<name>.md — pass --overwrite to replace it` | A run has already produced output at that path. | Re-run with `--overwrite` if you intend to replace it, or move/rename the existing file first if you want to keep it. |

Every one of these is a typed, stage-tagged error (audio, transcription,
cleanup, or output stage), so the error message on stderr always identifies
which stage failed. Failed runs never leave a partial Markdown file at the
destination, and no error message ever includes the API key, Base64 audio,
or full transcript contents.

## Privacy implications

This tool sends the maintainer's extracted audio and the raw transcript text
over the network to Xiaomi's MiMo V2.5 models (ASR for transcription, chat
for cleanup), under the account tied to `XIAOMI_API_KEY`. In plain terms:
the spoken content of the video leaves your local machine and is processed
by a third-party provider.

Do not run this tool on video content that shouldn't be shared with that
provider — confidential material, sensitive discussions, or recordings
involving people who haven't consented to their speech being sent to a
third-party AI service.

## Reviewing the generated Markdown before publishing

The cleanup stage is constrained to correcting spelling, punctuation,
capitalization, and grammar, and adding paragraph breaks — it must not
summarize, paraphrase, reorganize, censor, embellish, or add facts, headings,
or other unspoken structure. In practice, though, both stages can still
introduce errors: ASR can mishear words, and the cleanup model can
occasionally slip despite the constrained prompt. Read the output against
your own memory of the recording (or re-listen to it) before using it as
published or learning content on the website.

Every generated file already carries this reminder as a trailing note, so it
travels with the Markdown itself:

> This transcript was produced with model-assisted transcription and
> cleanup. It is not a verbatim or accessibility-certified transcript and
> requires human review before publishing.

Treat that note as accurate, not boilerplate — the cleaned result still
requires human review before it's used anywhere public.
