# W4 Checklist / Todos

Goal: execute week 4 in a clear order, starting with backend architecture, then video read, then video write, then the remaining foundations needed to turn this into a real product backend.

Scope for this file:

- backend only
- use this checklist as the main execution guide
- optimize for a smaller, working backend slice first

## Working rule with Codex 🤖

- [ ] Install and use Codex often for Rust, `ffmpeg`, `ffprobe`, API, architecture, and idiomaticity checks

## Phase 1 - Reshape backend architecture

Based on `W3_SUGGESTIONS_2.md`.

- [ ] Move the backend away from the global layered split toward `shared/` + `features/`
- [ ] Add `app_state.rs` and `router.rs`
- [ ] Keep `axum`, `sqlx::query_as!`, `tracing`, `utoipa`, JWT, migrations, and clear error handling
- [ ] Make sure health, auth, and OpenAPI still work after the refactor

Target backend shape:

```text
src/
├── main.rs
├── lib.rs
├── app_state.rs
├── router.rs
├── shared/
│   ├── config.rs
│   ├── cors.rs
│   ├── database.rs
│   ├── error.rs
│   ├── jwt.rs
│   ├── logging.rs
│   └── openapi.rs
└── features/
    ├── auth/
    └── system/
```

## Phase 2 - Add video read foundations

Goal: inspect media files reliably with `ffprobe` and expose useful read-only endpoints first.

- [ ] Create a `video_read` feature
- [ ] Implement `POST /video-inspections`
- [ ] Accept one uploaded video, run `ffprobe`, and return the inspected payload
- [ ] Return at least `original_file_name`, `file_size_bytes`, `duration_seconds`, `format_name`, `video_streams`, `audio_streams`, `width`, `height`, `fps`, codecs, bitrate, and audio stream count
- [ ] Read more `ffprobe` fields if useful; ask AI if unsure which ones matter
- [ ] Add clean error responses for empty upload, broken file, unsupported file, `ffprobe` timeout, and `ffprobe` process failure
- [ ] Add `video-read.http`
- [ ] Validate read behavior with the fixtures in `modules/backend/tests/fixtures/media`

Possible first endpoints:

```text
POST /video-inspections
```

Suggested internal files:

```text
features/video_read/
  mod.rs
  handlers.rs
  config.rs
  ffprobe_runner.rs
  ffprobe_mapper.rs
  types.rs
```

- `ffprobe_runner` runs the `ffprobe` command
- `ffprobe_mapper` turns raw `ffprobe` output into app-friendly types

## Phase 3 - Add video write foundations

Goal: support one real transform job through `ffmpeg`.

- [ ] Create a `video_write` feature
- [ ] Start with `compress`
- [ ] Implement `POST /video-jobs`
- [ ] Accept a video file input plus an `operation` like `compress` and the needed params such as codec, bitrate, or CRF
- [ ] Run one processing job, produce one output file, and return enough data to inspect the result
- [ ] Add `video-write-compress.http`
- [ ] Validate write behavior with the fixtures in `modules/backend/tests/fixtures/media`

Possible first write endpoints:

```text
POST /video-jobs
```

Suggested `POST /video-jobs` body shape:

```json
{
  "operation": "compress",
  "params": {
    "video_codec": "h264",
    "audio_codec": "aac",
    "crf": 23
  }
}
```

Alternative request shape if the endpoint accepts upload directly:

```text
multipart form-data
- file
- operation=compress
- params...
```

Suggested internal files:

```text
features/video_write/
  mod.rs
  handlers.rs
  config.rs
  ffmpeg_job_runner.rs
  types.rs
```
