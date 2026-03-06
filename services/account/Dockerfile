FROM rust:1.83.0-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates wget && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/account-service /usr/local/bin/account-service
EXPOSE 8080
CMD ["account-service"]
