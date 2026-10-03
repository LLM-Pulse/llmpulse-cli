FROM rust:1.85-bookworm@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4 AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --locked

FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171
LABEL org.opencontainers.image.title="llmpulse-cli"
LABEL org.opencontainers.image.description="CLI client for LLM Pulse AI visibility analytics"
LABEL org.opencontainers.image.url="https://llmpulse.ai"
LABEL org.opencontainers.image.documentation="https://llmpulse.ai/api-docs"
LABEL org.opencontainers.image.source="https://github.com/LLM-Pulse/llmpulse-cli"
LABEL org.opencontainers.image.vendor="LLM Pulse"

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/llmpulse /usr/local/bin/llmpulse
USER 65532:65532

ENTRYPOINT ["llmpulse"]
CMD ["--help"]
