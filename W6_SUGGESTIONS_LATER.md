# Backend suggestions — later (not in the diff)

Not defects from the last two weeks. These are the things that make a service which shells out to `ffmpeg`/`ffprobe`, can hold a request open for 300s, and is about to sit behind nginx on a single box, debuggable in prod. Roughly in priority order.

## 1. `CatchPanicLayer` so a panicking handler returns 500 instead of taking the worker down

`ffprobe_mapper` parses `ffprobe`'s JSON, `get_preset_by_name` parses the client's `operation` string. A stray `unwrap` / index / slice panic anywhere on that path currently unwinds the request task — with the default runtime that kills the response with no status, and a panic in a non-request task can abort the process.

**Fix:** add `tower_http::catch_panic::CatchPanicLayer` (feature `catch-panic`) near the top of the middleware stack in `router.rs`, mapping the panic to a fixed 500 body and a `tracing::error!` with the panic payload. One layer, isolates every handler.

```rust
use tower_http::catch_panic::CatchPanicLayer;

let app = Router::new()
    // ...routes...
    .layer(CatchPanicLayer::custom(handle_panic)); // outermost app-level layer

fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let detail = err
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| err.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown panic".to_string());
    tracing::error!(panic = %detail, "handler panicked");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "message": "Internal server error" })),
    )
        .into_response()
}
```

## 2. `#[tracing::instrument]` spans on the video pipeline + a request-id

`TraceLayer::new_for_http()` is in place (good), but every log line inside a request is unattributed — for a 300s `/api/video/process` call there's no way to see *which step* is slow or *which upload* a log line belongs to. The pipeline is already step-shaped: `read_multipart → ffprobe → ffmpeg → build_response`.

**Fix:**

1. `#[tracing::instrument(skip(...), fields(operation = %meta.command, input_bytes = written_bytes))]` on `process_video` / `inspect_video` and on the `ffmpeg_runner` / `ffprobe_runner` entry fns. Each step becomes a child span with its own timing; the ffmpeg progress log (W6 finding #4) then nests under the right span for free.
2. A request-id middleware: generate a UUID (or read an inbound `X-Request-Id`), put it in a root span (`tracing::info_span!("request", request_id = %id)`), echo it back as the `X-Request-Id` response header. Every downstream log line carries it. ~30 lines with `axum::middleware::from_fn`.

## 3. `/metrics` endpoint (Prometheus)

The service is CPU-bound — `ffmpeg` saturates cores — and there is currently no number to look at. Minimum useful set:

- `video_jobs_total{operation, status}` — counter, `status` in `ok|error|timeout`
- `video_process_duration_seconds{operation}` — histogram (per-step, wrapping the `ffmpeg` / `ffprobe` await)
- `video_input_bytes` — histogram
- rate-limiter rejections — counter (or scrape `tower_governor`'s own)

**Fix:** `metrics` + `metrics-exporter-prometheus`, install the recorder in `main`, add `GET /metrics` on the system router returning the encoded text. If auth stays off for the first release, keep `/metrics` off the public nginx vhost (LAN / basic-auth location only).

## Skip for now

Distributed tracing (OpenTelemetry / Jaeger) — one service, one box; the request-id + spans above cover it until there's a second service to correlate with.
