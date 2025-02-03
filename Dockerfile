# ---------- Builder Stage ----------
FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo install sqlx-cli
RUN sqlx database create

RUN cargo build --release

# ---------- Runtime Stage ----------
FROM debian:buster-slim

WORKDIR /app

COPY --from=builder /app/target/release/discord_bot ./

# Run the application
CMD ["./discord_bot"]