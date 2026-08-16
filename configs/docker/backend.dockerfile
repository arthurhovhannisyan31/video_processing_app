FROM rust:1.97 AS build-backend
SHELL ["/bin/bash", "-c"]

WORKDIR /home/server
COPY modules/backend .
COPY configs/scripts/backend-healthcheck.sh .

ENV SQLX_OFFLINE=true
RUN cargo build --release

# glibc compatible container
FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install ffmpeg -y \
    && apt-get install -y curl \
    && apt-get install -y bash \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /home/server
COPY --from=build-backend /home/server/backend-healthcheck.sh .
COPY --from=build-backend /home/server/target/release/video-processing-server .

ENTRYPOINT ["/home/server/video-processing-server"]