# Upload / processing flow review

Focused look at `POST /api/video/inspect` and `POST /api/video/jobs`: `routes.rs`, `process/form_data_reader.rs`, `helpers.rs`, `process/ffmpeg_runner.rs`, `process/build_response.rs`, `process/configs.rs`, `core/error.rs`.

**Short version:** the low-level Rust is idiomatic and in good shape. The main thing to revisit is the architecture: the handler runs synchronously, while the routes around it (`VIDEO_JOBS`, plus the declared `VIDEO_JOBS_BY_ID` / `VIDEO_JOBS_BY_ID_LOGS`) are already shaped for an async job model. The Cloudflare 120s connection cap and the websocket progress feature both point the same way.

## Already idiomatic (keep as is)

- Upload is streamed chunk by chunk to disk (`field.chunk()` loop into `tokio::fs::File`), no full-body buffering in RAM.
- Response is streamed back (`ReaderStream` + `Body::from_stream`) with `Content-Disposition: attachment`.
- `TempDir` for scratch space, `Command` with `kill_on_drop(true)` + `tokio::time::timeout`, ffmpeg args as `Vec<&str>` with no shell. No command injection.
- Upload filename is sanitised (`Path::new(name).file_name()` strips traversal) and unit-tested.
- Feature-module layout (`inspect/`, `process/`, runner / reader / mapper split) is clean and easy to follow.

## Architecture

### 1. Synchronous handler for a job that can run up to 300s

`process_video` keeps the HTTP connection open for the whole ffmpeg run and streams the output file back in the same response:

```rust
pub async fn process_video(media_data: Multipart) -> Result<impl IntoResponse, ApplicationError> {
  let temp_dir = TempDir::new().map_err(ServerError::IO)?;
  let ProcessVideoMeta { command, file_path } =
    process::form_data_reader::read(media_data, temp_dir.path()).await?;
  let output_path = append_path_suffix(&file_path, OUTPUT_PATH_SUFFIX)?;
  let preset = get_preset_by_name(&command)?;
  ffmpeg_runner(&file_path, &output_path, preset).await?; // blocks up to 300s
  Ok(build_response(&file_path, &output_path).await?)
}
```

The route is `routes::VIDEO_JOBS`, the OpenAPI body is shaped like a job API, and `VIDEO_JOBS_BY_ID` / `VIDEO_JOBS_BY_ID_LOGS` are already declared. The async model is the natural next step, and it is also what the websocket progress feature needs underneath it. Disabling the Cloudflare proxy works around the 120s cap for now, but the same request will keep outliving normal proxy timeouts.

**Suggested direction (matches where the routes already point):**

- `POST /api/video/jobs`: validate input, persist a `video_jobs` row (`status = queued`), enqueue or spawn the work, return `202 Accepted` with `{ "job_id": ... }`.
- `GET /api/video/jobs/:id`: job status + result metadata (original bytes, output bytes, ratio).
- `GET /api/video/jobs/:id/result`: stream the output file, with `Content-Length` set.
- progress: `GET /api/video/jobs/:id/logs` (SSE) or a websocket, fed by the ffmpeg `-progress` parser that currently only writes to server logs.

This is a sizeable change and does not need to happen for the current release. Worth planning before the websocket work starts, since that work depends on it.

### 2. No job persistence

There is no `video_jobs` table yet, so a job leaves no record: no history, no resume after a dropped connection, no per-user quota, no audit. Migration infra is already in place (`_sqlx_migrations`, one users migration), so this is mostly a new migration plus a small repo. Once the table exists, a transaction with `SELECT ... FOR UPDATE` around the status update is the pattern for moving a job through `queued -> running -> done / failed` safely.

## Smaller items

### 3. `temp_dir` drops before the response finishes streaming

```rust
  Ok(build_response(&file_path, &output_path).await?)
  // temp_dir (TempDir guard) drops here, deleting the directory and files
  // while the streaming Body is still being sent to the client
```

`build_response` opens the output file, builds a streaming `Body`, and returns. The `TempDir` guard owned by `process_video` then drops at the end of the function, removing the directory while the body is still streaming. It currently works because on Linux `File::open` already holds an fd, so `unlink` only removes the name and the bytes stay readable until the fd closes. On an early return, a panic, or a non-Linux target it would leak or fail, and the lifetime is not obvious from reading the code.

**Fix:** either read the output file fully before returning (fine at this size ceiling), or move the `TempDir` into the stream's state so it drops after the last chunk. A small `struct StreamWithGuard { inner, _dir: TempDir }` that implements `Stream` is the clean version.

### 4. No `Content-Length` on the streamed response

`Body::from_stream` sends a chunked response with no length, so the browser cannot show a progress bar for the download. Since the UI is adding progress indicators, this is an easy win:

```rust
let len = tokio::fs::metadata(output_path).await.map_err(ServerError::IO)?.len();
Response::builder()
  .header(header::CONTENT_TYPE, "video/mp4")
  .header(header::CONTENT_LENGTH, len)
  .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{original_name}\""))
  .body(Body::from_stream(stream))
```

### 5. ffmpeg failure on bad input returns 500

```rust
// ffmpeg_runner.rs
if !status.success() {
  return Err(ServerError::Processing(format!("ffmpeg error: {}", status)));
}
```

`ServerError::Processing` maps to `ApplicationError::Internal`, so a 500. A valid request carrying a corrupt or unsupported file is a client problem and fits 422 (or 400) better. Keeping it as 500 mixes real server faults with bad uploads in the error-rate metrics.

**Fix:** separate "ffmpeg could not spawn / timed out / crashed" (500) from "ffmpeg exited non-zero on the input" (422), e.g. a `ServerError::UnprocessableInput(String)` variant mapping to `ApplicationError::UnprocessableEntity`.

### 6. `operation` is parsed twice and carried as a `String`

`process/form_data_reader.rs` parses the field *name* into `FieldName`, but stores the operation *value* as raw text:

```rust
FieldName::Operation => {
  meta.command = field.text().await?; // String
}
```

then `process_video` parses it again via `get_preset_by_name(&command)` -> `OperationType::from_str`. Two parse sites and an untyped `String` on `ProcessVideoMeta`.

**Fix:** validate into the enum in the reader, store `operation: OperationType` on `ProcessVideoMeta`, and have `get_preset_by_name` take `OperationType`.

### 7. Multipart field order lets a bad request upload the file first

Fields are consumed in the order the client sends them. `video` before `operation` means the whole file is written to disk before `operation` is checked, so a caller can push `video_max_body_size` bytes with `operation=garbage`.

**Fix:** once `operation` is known invalid, return before consuming the video field, or read non-file fields first and reject before the first `read_video_to_file`. Client order cannot be forced, but the request can be refused early.

### 8. `-progress pipe:2` shares stderr with ffmpeg's diagnostic output

```rust
cmd.stderr(Stdio::piped());
// preset: ... "-progress", "pipe:2"
```

The progress key/value stream shares fd 2 with ffmpeg's normal logging, so the parser in `log_task` reads both interleaved, and real error lines are only visible if a non-progress branch logs them (currently none does, see W6 finding #4).

**Fix:** use `-progress pipe:1` with stdout piped for the parser and stderr piped separately for errors, or `-progress unix:/path` to a dedicated socket.

## Priority

1. #1 is the root item (async job model); #2 (persistence) is part of the same change. Plan before the websocket work; not required for this release.
2. #3, #4, #5 are independent and safe to do first.
3. #6, #7, #8 are cleanups.
