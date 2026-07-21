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

Main layout:

```text
+----------------------------------------------------------------------------------+
| Video Processing Studio                                          [New Job]       |
+----------------------------------------------------------------------------------+
| Sidebar          | Main Panel                                  | Inspector       |
|------------------|---------------------------------------------|-----------------|
| Library          | Selected Video / Preview                    | Job Config      |
| Jobs             |---------------------------------------------|-----------------|
| Presets          | Metadata                                    | Preset          |
| Outputs          | Queue snapshot                              | Codec           |
|                  | Recent activity                             | Resolution      |
|                  |                                             | Bitrate         |
|                  |                                             | Audio           |
|                  |                                             | [Run Job]       |
+----------------------------------------------------------------------------------+
| Bottom Drawer: logs / ffmpeg output / progress / errors                           |
+----------------------------------------------------------------------------------+
```

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
