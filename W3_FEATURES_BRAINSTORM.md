# W3 Features Brainstorm

## Goal

Build a useful video processing product that shows strong Rust capability, with a polished frontend layer to showcase the product clearly.

## Direction

Build a modern video processing app powered by Rust, `ffmpeg`, and `ffprobe`.

The backend should be the core value of the project.  
The frontend should make the product easy to understand and easy to demo.

## Core product

The app should let users:

- upload video files
- preview media
- inspect metadata
- run processing jobs
- track progress
- retrieve outputs

## MVP features

- upload one or multiple videos
- preview a selected video
- inspect metadata
  duration, resolution, fps, codecs, bitrate, file size, aspect ratio
- compress video
- transcode with presets
- trim by start/end
- resize to target resolution
- extract audio
- generate thumbnails
- queue jobs
- show `queued / running / done / failed`
- show progress and logs
- show outputs and input/output comparison

## Why this fits the project

- useful real-world product
- strong Rust showcase
- clear backend value
- enough frontend to present the product well
- easier to scope than a complex editor

## Rust-heavy value

- `ffmpeg` orchestration
- `ffprobe` metadata parsing
- job system
- progress tracking
- concurrent processing
- output management
- structured logging
- strong error handling

## UI direction

The product should feel like a control room for video processing, not a classic timeline editor.

Primary user flow:

`select input -> inspect -> configure job -> run -> monitor -> retrieve output`

Recommended main layout:

```text
+--------------------------------------------------------------------------------------------------+
| Video Processing Studio                                             Search        [New Job]      |
+--------------------------------------------------------------------------------------------------+
| Nav Rail        | Main Workspace                                                     | Inspector |
|-----------------|--------------------------------------------------------------------|-----------|
| Library         | Selected Asset                                                     | Job Draft |
| Jobs            |--------------------------------------------------------------------|-----------|
| Presets         | Preview                                                            | Preset    |
| Outputs         |                                                                    | Codec     |
|                 | Metadata summary                                                   | Size      |
|                 | duration • resolution • fps • codecs • bitrate • file size         | Audio     |
|                 |                                                                    | Output    |
|                 | Recent jobs for this asset                                          | [Run Job] |
|                 | queued • running • done • failed                                    |           |
+--------------------------------------------------------------------------------------------------+
| Bottom Drawer                                                                               Logs |
| ffprobe streams parsed | ffmpeg command | progress events | warnings | errors                    |
+--------------------------------------------------------------------------------------------------+
```

Why this should be the main layout:

- it matches the product brief better than a creative editing layout
- it makes the Rust backend visible through jobs, logs, and progress
- it is easier to understand in a demo
- it scales naturally from single-file processing to batch workflows

Layout rules:

- the left rail is for product sections, not editing tools
- the center panel is the primary reading area and changes by context
- the right inspector is always actionable and should contain the current form or status details
- the bottom drawer is persistent because logs are part of the product value, not a hidden debug view

Workspace states:

Library-focused state:

```text
+--------------------------------------------------------------------------------------------------+
| Library                                                                                          |
+--------------------------------------------------------------------------------------------------+
| [video_a.mp4] [video_b.mov] [podcast_ep01.mp4] [teaser_cut.mp4]                                 |
|                                                                                                  |
| Selecting an asset opens preview + metadata + quick actions in the main workspace               |
+--------------------------------------------------------------------------------------------------+
```

Jobs-focused state:

```text
+--------------------------------------------------------------------------------------------------+
| Jobs                                                                                             |
+--------------------------------------------------------------------------------------------------+
| #184  running   transcode web_1080p         72%   input_interview.mp4                           |
| #183  done      extract audio               100%  podcast_ep01.mp4                              |
| #182  failed    subtitle burn-in            error intro.mov                                      |
| #181  queued    thumbnail generation         0%   teaser_cut.mp4                                |
+--------------------------------------------------------------------------------------------------+
```

Batch-focused state:

```text
+--------------------------------------------------------------------------------------------------+
| Batch Queue                                                                       [Run Batch]    |
+--------------------------------------------------------------------------------------------------+
| Active Jobs                                                                                      |
| [running] podcast_ep01.mp4   Compress 1080p        64%                                           |
| [running] podcast_ep02.mp4   Compress 1080p        41%                                           |
| [queued ] podcast_ep03.mp4   Compress 1080p         0%                                           |
| [queued ] podcast_ep04.mp4   Compress 1080p         0%                                           |
|                                                                                                  |
| Summary: 6 files • 2 running • 2 queued • 1 done • 1 failed                                     |
+--------------------------------------------------------------------------------------------------+
```

Mobile / narrow-screen adaptation:

```text
+-----------------------------------------------------------+
| Video Processing Studio                    [New Job]       |
+-----------------------------------------------------------+
| Tabs: Library | Jobs | Presets | Outputs                  |
+-----------------------------------------------------------+
| Preview / Selected content                                |
| Metadata                                                  |
| Job config                                                |
| [Run Job]                                                 |
+-----------------------------------------------------------+
| Collapsible Logs Drawer                                   |
+-----------------------------------------------------------+
```

Important product decision:

The timeline should not be the main container for this product.
If a timeline exists, it should appear only in specialized flows like trim or clip extraction.
The default shell should remain centered on assets, jobs, and processing status.

Compression job:

```text
+---------------------------------------------------------------+
| Compression Job                                               |
+---------------------------------------------------------------+
| Input: interview_master.mov                                   |
| Size: 2.8 GB                                                  |
|                                                               |
| Goal                                                          |
| (o) Smaller file                                              |
| ( ) Balanced                                                  |
| ( ) Best quality                                              |
|                                                               |
| Codec                                                         |
| (o) H.264                                                     |
| ( ) H.265                                                     |
| ( ) AV1                                                       |
|                                                               |
| Resolution                                                    |
| (o) Keep original                                             |
| ( ) 1080p                                                     |
| ( ) 720p                                                      |
|                                                               |
| Audio                                                         |
| (o) AAC 128k                                                  |
| ( ) AAC 192k                                                  |
| ( ) Copy                                                      |
|                                                               |
| Estimated Output                                              |
| ~ 640 MB                                                      |
|                                                               |
|                           [Compress Video]                    |
+---------------------------------------------------------------+
```

Jobs and logs:

```text
+----------------------------------------------------------------------------------+
| Jobs                                                                              |
+----------------------------------------------------------------------------------+
| Job #184  running   transcode vertical_social     72%                             |
| Job #183  done      extract audio                 100%                            |
| Job #182  failed    subtitle burn-in              error                           |
|                                                                                  |
| Logs for Job #184                                                                 |
| ffprobe parsed streams                                                            |
| ffmpeg started                                                                    |
| filter: scale=1080:1920                                                           |
| audio: aac 128k                                                                   |
| progress: frame=4212 time=00:02:48                                                |
+----------------------------------------------------------------------------------+
```

Batch processing view:

```text
+--------------------------------------------------------------------------------------------------+
| Video Processing Studio                                                         [New Batch Job]  |
+--------------------------------------------------------------------------------------------------+
| Sidebar          | Batch Queue                                                    | Inspector    |
|------------------|----------------------------------------------------------------|--------------|
| Library          | Active Jobs                                                     | Batch Config |
| Jobs             |----------------------------------------------------------------|--------------|
| Presets          | [running] podcast_ep01.mp4   Compress 1080p        64%         | Preset       |
| Outputs          | [running] podcast_ep02.mp4   Compress 1080p        41%         | Web 1080p    |
|                  | [queued ] podcast_ep03.mp4   Compress 1080p         0%         |              |
|                  | [queued ] podcast_ep04.mp4   Compress 1080p         0%         | Parallelism  |
|                  | [done   ] teaser_cut.mp4     Audio extract        100%         | 2 workers    |
|                  | [failed ] intro.mov          Thumbnail gen        error        |              |
|                  |                                                                | Output Path  |
|                  | Batch Summary                                                  | /exports/... |
|                  | 6 files   2 running   2 queued   1 done   1 failed             |              |
|                  |                                                                | [Run Again]  |
+--------------------------------------------------------------------------------------------------+
| Logs                                                                                             |
| [podcast_ep01.mp4] ffmpeg started                                                                |
| [podcast_ep02.mp4] progress: frame=1822 time=00:01:12                                            |
| [intro.mov] error: invalid input stream                                                          |
+--------------------------------------------------------------------------------------------------+
```

## Main product message

**A modern video processing app powered by Rust, `ffmpeg`, and `ffprobe`.**

## Suggested feature implementation order

Because `ffmpeg` work is still new territory, the project should start with a backend-first milestone before building the full product shell.

The goal of this first phase is to make the essential `ffmpeg` and `ffprobe` workflows visible and understandable with as little UI as possible.

### Phase 0 - Backend-only `ffmpeg` lab

Build a small backend or CLI slice that proves the core media pipeline works.

Start with:

- inspect one input file with `ffprobe`
- parse metadata into a clean Rust structure
- run one processing job with `ffmpeg`
- track job status
- capture useful logs
- write output files in a predictable place

Suggested first backend capabilities:

- inspect video metadata
  duration, resolution, fps, codecs, bitrate, audio streams, file size
- run one simple job
  good first choices: `compress`, `trim`, or `extract audio`
- expose progress
  `queued / running / done / failed`
- persist job result info
  input path, output path, started at, finished at, error message, output metadata

This phase does not need a polished frontend.
A minimal API, CLI, or debug page is enough if it helps the team understand the pipeline clearly.

### Why this order is useful

- it reduces the number of unknowns at the start
- it forces the team to understand `ffprobe` and `ffmpeg` before building too much UI
- it reveals the real backend problems early
- it makes the later frontend work much easier because the job model is already clearer

### Suggested order

1. File input handling
   Accept a local file and validate that the backend can read it safely.
2. Metadata inspection with `ffprobe`
   Return a structured object for duration, streams, codecs, bitrate, resolution, and file size.
3. First processing job
   Implement one simple transform such as `compress` or `extract audio`.
4. Progress tracking
   Capture process progress and map it into app-friendly job states.
5. Logging and errors
   Separate useful execution logs from noisy raw output and store failure reasons clearly.
6. Output management
   Save outputs with predictable naming and keep input/output metadata together.
7. Minimal API surface
   Add simple endpoints or commands for `inspect`, `run job`, `get job status`, and `get logs`.
8. `.http` verification files for `ffmpeg` / `ffprobe`
   Add request files focused only on media inspection and processing flows so the team can verify the returned data without a frontend.
9. Simple frontend shell
   Only after the backend slice feels stable, connect the product layout to it.

### Good minimal API shape

```text
POST /inspect
POST /jobs/compress
GET  /jobs/:id
GET  /jobs/:id/logs
GET  /jobs/:id/output
```

### `.http` files for backend-first verification

It is useful to keep a small set of `.http` files dedicated only to `ffmpeg` and `ffprobe` validation.

These files should not be about auth or generic backend plumbing.
They should exist only to verify media pipeline behavior and returned data.

Good candidates:

- `inspect.http`
  verify `ffprobe` metadata shape and stream parsing
- `jobs-compress.http`
  start a compression job and check returned job identifiers and initial status
- `jobs-extract-audio.http`
  start an audio extraction job and verify output metadata
- `jobs-status.http`
  inspect job progression and final states
- `jobs-logs.http`
  inspect useful execution logs and failure messages

Why this helps:

- it lets the team validate `ffprobe` and `ffmpeg` behavior without waiting for frontend work
- it makes the backend contract visible very early
- it helps confirm that the returned data is actually the data the product needs
- it gives a lightweight debugging workflow while learning the media pipeline

### Important backend lessons to learn early

- how to invoke `ffprobe` safely and parse its output cleanly
- how to build `ffmpeg` commands without brittle string handling
- how to read progress from a long-running process
- how to classify process failures clearly
- how to prevent long jobs from blocking the rest of the backend
- how to organize outputs so later retrieval is simple

### Product implication

The frontend should not outrun the backend learning curve.
If the backend cannot yet inspect, run, track, and report jobs clearly, the UI should stay minimal until those fundamentals are stable.
