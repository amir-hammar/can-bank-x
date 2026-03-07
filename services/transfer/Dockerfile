FROM rust:1.83.0-bookworm AS deps
WORKDIR /app
COPY Cargo.toml Cargo.lock* .
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

FROM rust:1.83.0-bookworm AS builder
WORKDIR /app
COPY --from=deps /app/target target
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/transfer-service /usr/local/bin/transfer-service
EXPOSE 8080
CMD ["transfer-service"]
