FROM rust:1.90-slim-bookworm AS builder

WORKDIR /push-platform

RUN apt-get update  \
    && apt-get install -y pkg-config libssl-dev  \
    && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml ./
COPY backend/src ./src
COPY backend/migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /push-platform

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /push-platform/target/release/push-platform ./push-platform

USER 1000

CMD ["./push-platform"]
