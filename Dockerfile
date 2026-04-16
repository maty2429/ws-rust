# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.87-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV CARGO_BUILD_TARGET="x86_64-unknown-linux-musl"

# Cachear dependencias antes de copiar el src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf src

# Compilar el codigo real
COPY src ./src
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/websocket /websocket

EXPOSE 3001

CMD ["/websocket"]
