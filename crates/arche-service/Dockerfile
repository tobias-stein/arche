FROM rust:1.88.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
COPY bench/ bench/
COPY arche-service/migrations/ arche-service/migrations/

RUN cargo build -p arche-service --release && \
    cp target/release/arche-service /arche-service

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /arche-service /usr/local/bin/arche-service

EXPOSE 8080
ENV ARCHE_PORT=8080

CMD ["arche-service"]
