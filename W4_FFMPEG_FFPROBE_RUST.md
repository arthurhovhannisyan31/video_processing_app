# FFmpeg / FFprobe / Rust

## Goal

Have one practical reference for the backend side of the project:

- what `ffmpeg` does
- what `ffprobe` does
- what data we usually need
- how to call both from Rust
- when a crate helps and when plain process spawning is enough

This guide is intentionally backend-first.
The point is not to learn every corner of multimedia engineering.
The point is to become productive fast on the app we want to build.

## The short mental model

`ffprobe` inspects media.

- it reads a file or stream
- it reports metadata
- it does not transform media
- it is usually the first step in the pipeline

`ffmpeg` transforms media.

- it decodes inputs
- it applies mapping, trimming, scaling, transcoding, extraction, filtering
- it writes outputs
- it is usually the job engine of the product

Very short product flow:

```text
input file
  -> ffprobe
  -> parse metadata into Rust structs
  -> choose a job config
  -> ffmpeg
  -> capture progress + logs + result
  -> store output metadata
```

## What we usually need from `ffprobe`

For this product, the essential metadata is:

- file path
- file size
- duration
- container format
- video streams
- audio streams
- width / height
- frame rate
- video codec
- audio codec
- bitrate
- aspect ratio

In practice, `ffprobe` should usually be called in JSON mode.
That is the easiest shape to parse safely in Rust.

Typical command:

```bash
ffprobe \
  -v error \
  -show_format \
  -show_streams \
  -of json \
  input.mp4
```

Why this shape:

- `-v error` reduces noise
- `-show_format` gives container-level data
- `-show_streams` gives stream-level data
- `-of json` makes the output machine-friendly

## What we usually need from `ffmpeg`

For an MVP backend, the most useful transforms are simple:

- compress video
- transcode with a preset
- trim by start / end
- resize
- extract audio
- generate thumbnails

For each job, the backend usually needs to know:

- the exact input file
- the chosen operation
- the chosen parameters
- the output path
- whether the job started
- whether the job is still running
- whether it succeeded or failed
- what progress has been observed
- what useful logs or errors were produced

## Important practical rule

For this project, the simplest and best first approach is:

**Call the `ffmpeg` and `ffprobe` binaries from Rust.**

Why:

- much faster to get working
- much easier to debug
- much easier for a team still learning the media stack
- aligns well with backend job orchestration
- lets you use official CLI behavior directly

For this app, Rust is mainly orchestrating jobs, parsing outputs, tracking progress, and managing errors.
That is already strong backend work.

## How this usually looks in Rust

There are two practical approaches.

### 1. Spawn the CLI directly

Use `std::process::Command` or `tokio::process::Command`.

This is the recommended starting point.

Good for:

- `ffprobe` metadata inspection
- basic `ffmpeg` jobs
- progress tracking
- logs
- deterministic command construction

### 2. Use a CLI-oriented helper crate

Useful if you want a bit more ergonomics around the binaries.

The most relevant current option I found is `ffmpeg-sidecar` (`2.5.2` as crawled today on Docs.rs), which wraps a standalone FFmpeg binary and can also auto-download FFmpeg if needed.

Good for:

- easier interaction with the CLI
- environments where bundling or downloading the binary is useful
- richer process/event handling than raw `Command`

Less necessary if:

- the backend already runs in an environment where `ffmpeg` and `ffprobe` are installed
- plain `tokio::process::Command` already covers the needed flows

## Recommended stack for this backend

Given the current backend stack already uses `axum`, `tokio`, `serde`, `anyhow`, `thiserror`, and `tracing`, a pragmatic media stack is:

- `tokio::process::Command`
- `serde` / `serde_json`
- `tracing`
- `anyhow` and/or `thiserror`
- plain Rust structs for parsed metadata and job status

That is enough to support a first API such as:

- `POST /inspect`
- `POST /jobs/compress`
- `GET /jobs/:id`
- `GET /jobs/:id/logs`

## Why this is better for this project

For this project specifically, CLI orchestration is a better starting point than direct FFmpeg bindings.

Why:

- the team is still building confidence with `ffmpeg` and `ffprobe`
- the product needs job orchestration more than custom frame decoding
- backend work here is mostly process management, metadata parsing, progress tracking, and error handling
- the CLI is the most direct path to a working backend slice
- debugging a failed command is much easier than debugging FFI behavior

This means the backend should focus on:

- building safe command arguments
- running processes with timeouts
- capturing stdout and stderr cleanly
- mapping external output into domain structs
- storing job state and outputs predictably

The important idea is:

**For this product, Rust is orchestrating media tools, not replacing FFmpeg internals.**

That is already a strong and legitimate backend architecture.

## Useful backend pattern

The most useful ideas here are structural, not library-specific.

The good pattern is:

```text
runtime config
  -> runner
  -> parser / mapper
  -> domain model
  -> API response
```

Concretely, this pattern does a few things right:

- configurable `ffmpeg` / `ffprobe` binary paths
- configurable timeouts
- `tokio::process::Command`
- `kill_on_drop(true)` for subprocess cleanup
- one layer that runs the external process
- one layer that parses raw output
- one layer that maps into app-specific data

That is a very good shape to copy in a smaller form here.

Recommended file structure for this project:

```text
media/
  runtime_config.rs
  ffprobe_runner.rs
  ffprobe_mapper.rs
  ffmpeg_job_runner.rs
  types.rs
```

The responsibilities should stay strict:

- `runtime_config.rs`
  binary names, temp dirs, timeout values
- `ffprobe_runner.rs`
  only runs `ffprobe`
- `ffprobe_mapper.rs`
  only parses and maps probe output
- `ffmpeg_job_runner.rs`
  only runs transform jobs
- `types.rs`
  shared domain structs

This separation matters because it keeps media logic understandable while the team is still learning the stack.

## Simple `ffprobe` example in Rust

This is the basic pattern:

1. run `ffprobe`
2. capture stdout
3. deserialize JSON
4. map it into domain structs

```rust
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    index: u32,
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    filename: String,
    duration: Option<String>,
    size: Option<String>,
    bit_rate: Option<String>,
    format_name: Option<String>,
}

fn inspect_media(path: &str) -> Result<FfprobeOutput> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_format",
            "-show_streams",
            "-of",
            "json",
            path,
        ])
        .output()
        .context("failed to spawn ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed: {stderr}");
    }

    let parsed: FfprobeOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse ffprobe json")?;

    Ok(parsed)
}
```

What matters here:

- do not parse human text if JSON is available
- treat `stderr` as useful error context
- deserialize into an intermediate shape first
- map later into your own domain model

## Better domain model after parsing

Usually you do not want raw `ffprobe` JSON to spread everywhere in the codebase.

Instead, map into your own structs:

```rust
#[derive(Debug)]
struct MediaMetadata {
    path: String,
    duration_seconds: Option<f64>,
    file_size_bytes: Option<u64>,
    video_streams: Vec<VideoStream>,
    audio_streams: Vec<AudioStream>,
}

#[derive(Debug)]
struct VideoStream {
    codec: Option<String>,
    width: u32,
    height: u32,
    fps: Option<f64>,
}

#[derive(Debug)]
struct AudioStream {
    codec: Option<String>,
}
```

Reason:

- easier to validate
- easier to evolve
- easier to expose through API responses
- protects the rest of the backend from raw external output shape changes

## Simple `ffmpeg` example in Rust

Very first compression example:

```rust
use std::process::Command;

use anyhow::{Context, Result};

fn compress_video(input: &str, output: &str) -> Result<()> {
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            input,
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            output,
        ])
        .status()
        .context("failed to spawn ffmpeg")?;

    if !status.success() {
        anyhow::bail!("ffmpeg exited with non-zero status");
    }

    Ok(())
}
```

This is enough for a first backend slice.
It is not enough for a full job runner, because you still need:

- progress tracking
- log capture
- cancellation strategy
- timeout strategy
- job persistence

## The essential progress trick

Do not rely only on the default human-readable stderr for progress if you can avoid it.

The official `ffmpeg` CLI supports:

```bash
ffmpeg -progress pipe:1 -i input.mp4 output.mp4
```

That emits key/value progress events that are easier to parse than free-form log text.

Typical shape:

```text
frame=123
fps=49.5
bitrate=512.3kbits/s
total_size=123456
out_time_ms=2000000
out_time=00:00:02.000000
speed=1.25x
progress=continue
...
progress=end
```

For Rust, this is usually the right pattern:

- read `stdout` line by line for progress events
- read `stderr` line by line for warnings and errors
- convert progress into your app status model

## Async `ffmpeg` example with progress

This fits our `tokio` backend better than blocking `std::process::Command`.

```rust
use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

async fn run_ffmpeg_with_progress(input: &str, output: &str) -> Result<()> {
    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-nostdin",
            "-progress",
            "pipe:1",
            "-i",
            input,
            "-c:v",
            "libx264",
            "-c:a",
            "aac",
            output,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn ffmpeg")?;

    let stdout = child.stdout.take().context("missing ffmpeg stdout")?;
    let stderr = child.stderr.take().context("missing ffmpeg stderr")?;

    let mut progress_lines = BufReader::new(stdout).lines();
    let mut error_lines = BufReader::new(stderr).lines();

    let progress_task = tokio::spawn(async move {
        while let Some(line) = progress_lines.next_line().await? {
            println!("progress: {line}");
        }
        Ok::<(), anyhow::Error>(())
    });

    let stderr_task = tokio::spawn(async move {
        while let Some(line) = error_lines.next_line().await? {
            eprintln!("ffmpeg stderr: {line}");
        }
        Ok::<(), anyhow::Error>(())
    });

    let status = child.wait().await?;
    progress_task.await??;
    stderr_task.await??;

    if !status.success() {
        anyhow::bail!("ffmpeg exited with non-zero status");
    }

    Ok(())
}
```

Notes:

- `-nostdin` avoids weird interactive behavior in backend jobs
- `stdout` becomes structured progress
- `stderr` still matters for warnings and failures
- later, replace `println!` / `eprintln!` with `tracing` and persistent job logs

## Parsing progress into app state

You do not need a perfect parser on day one.

A simple parser for `key=value` lines is enough:

```rust
fn parse_progress_line(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once('=')?;
    Some((key.trim(), value.trim()))
}
```

Then update a struct:

```rust
#[derive(Debug, Default)]
struct JobProgress {
    frame: Option<u64>,
    fps: Option<f64>,
    total_size: Option<u64>,
    out_time_ms: Option<u64>,
    speed: Option<String>,
    done: bool,
}
```

Then interpret the values simply:

- `progress=continue` means still running
- `progress=end` means ffmpeg reached the end
- process exit status still matters for true success / failure

## Common job examples

### 1. Inspect metadata

Use `ffprobe`.

```bash
ffprobe -v error -show_format -show_streams -of json input.mp4
```

### 2. Compress video

```bash
ffmpeg -y -i input.mp4 -c:v libx264 -preset medium -crf 23 -c:a aac -b:a 128k output.mp4
```

### 3. Extract audio

```bash
ffmpeg -y -i input.mp4 -vn -c:a aac -b:a 128k output.m4a
```

### 4. Trim clip

```bash
ffmpeg -y -ss 00:00:05 -to 00:00:15 -i input.mp4 -c:v libx264 -c:a aac output.mp4
```

### 5. Resize

```bash
ffmpeg -y -i input.mp4 -vf scale=1280:720 -c:v libx264 -c:a aac output_720p.mp4
```

## Common mistakes to avoid

### 1. Parsing the wrong output

Bad:

- trying to scrape the default human log format for everything

Better:

- use `ffprobe` JSON
- use `ffmpeg -progress pipe:1` for progress
- keep `stderr` mainly for diagnostics

### 2. Building commands as one fragile shell string

Bad:

- `"ffmpeg -i " + input + " ..."`

Better:

- pass arguments as distinct items in `Command::args`

Reason:

- safer with spaces
- safer with escaping
- easier to log and test

### 3. Trusting input blindly

Always expect:

- missing audio streams
- multiple audio streams
- broken files
- unsupported codecs
- empty duration
- inconsistent metadata

This is why the generated test fixtures matter.

## Test fixtures we already have

The backend now has media fixtures under:

[`modules/backend/tests/fixtures/media`](</Users/user/dev/2-RUST-COACHING/arthur-hovanisan/video_processing_app/modules/backend/tests/fixtures/media>)

Useful cases:

- `sample_av.mp4`
- `vertical_no_audio.mp4`
- `dual_audio_tracks.mp4`
- `audio_only.m4a`
- `broken_truncated.mp4`

These are enough to start validating:

- stream parsing
- aspect ratio handling
- no-audio edge cases
- multi-audio edge cases
- broken-input error handling

## Where a crate may help later

### Probably useful later

- `ffmpeg-sidecar`
  if we want more ergonomic binary management or richer CLI wrapping

## Suggested implementation order

1. Add a small media inspection service around `ffprobe`
2. Return parsed metadata as clean JSON from the backend
3. Add one `ffmpeg` job, likely `compress` or `extract audio`
4. Add progress parsing through `-progress pipe:1`
5. Persist job state and logs
6. Validate behavior using the media fixtures and `.http` requests

## Good first boundary

If we can do these three things reliably, we are in a very good place:

1. inspect a real media file
2. run one real transform
3. report progress and failure clearly

Everything else can build on top of that.

## References

Checked on July 21, 2026.

- FFprobe documentation: https://ffmpeg.org/ffprobe.html
- FFmpeg documentation: https://ffmpeg.org/ffmpeg.html
- `ffmpeg-sidecar` docs/source: https://docs.rs/crate/ffmpeg-sidecar/latest
- `ffmpeg-sidecar` GitHub: https://github.com/nathanbabcock/ffmpeg-sidecar
