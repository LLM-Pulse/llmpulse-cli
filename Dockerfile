FROM rust:1.99-bookworm@sha256:e6f2cff35d0f309acf8433b4f05878ed77092602c5065cc81801177da0bb3a9c AS builder
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
RUN cargo build --release --locked

FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3
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
