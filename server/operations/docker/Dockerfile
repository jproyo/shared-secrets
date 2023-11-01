# syntax=docker/dockerfile:1.3-labs
FROM rust:buster AS builder
WORKDIR /app
RUN apt update -y
RUN apt install -y cmake protobuf-compiler

COPY Cargo.lock /app
RUN echo "[workspace]\nmembers=[\"sss-wrap\", \"server\"]" >> /app/Cargo.toml
COPY sss-wrap /app/sss-wrap
RUN cargo new --lib /app/server
COPY server/Cargo.toml /app/server
RUN --mount=type=cache,target=/usr/local/cargo/registry cd /app/server && cargo build -p shared-secret-server --release

COPY server /app/server
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
set -e
touch /app/server/src/lib.rs
cd /app/server
cargo build --release -p shared-secret-server
EOF

RUN mv /app/target/release/shared-secret-server /app

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR /app
RUN apt update -y
RUN apt install -y ca-certificates protobuf-compiler
COPY --from=builder /app/shared-secret-server /app
ENTRYPOINT ["/app/shared-secret-server"]
