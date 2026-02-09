FROM rust:1.75-slim

WORKDIR /app

RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "run", "-w", "src", "-w", "Cargo.toml"]
