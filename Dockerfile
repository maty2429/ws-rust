# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.87-alpine AS builder

ARG TARGETARCH

RUN apk add --no-cache musl-dev
WORKDIR /app

# Cachear dependencias antes de copiar el src
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY proto ./proto
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN case "$TARGETARCH" in \
      "amd64") export RUST_TARGET="x86_64-unknown-linux-musl" ;; \
      "arm64") export RUST_TARGET="aarch64-unknown-linux-musl" ;; \
      *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    rustup target add "$RUST_TARGET" && \
    RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "$RUST_TARGET"
RUN rm -rf src

# Compilar el codigo real
COPY src ./src
RUN case "$TARGETARCH" in \
      "amd64") export RUST_TARGET="x86_64-unknown-linux-musl" ;; \
      "arm64") export RUST_TARGET="aarch64-unknown-linux-musl" ;; \
      *) echo "Unsupported TARGETARCH: $TARGETARCH" && exit 1 ;; \
    esac && \
    touch src/main.rs && \
    RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target "$RUST_TARGET" && \
    cp "target/$RUST_TARGET/release/websocket" /websocket

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM scratch

COPY --from=builder /websocket /websocket

EXPOSE 3001
EXPOSE 50051

CMD ["/websocket"]
