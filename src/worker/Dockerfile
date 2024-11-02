FROM rust:1.82-bullseye AS builder

WORKDIR /app

# Build dependencies
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo build --release

# Build app
# RUN cargo build --release
# RUN cargo install --path .

# Use smaller base image for final build
FROM debian:bullseye-slim
COPY --from=builder /app/target/release/worker /usr/local/bin/worker

CMD ["worker"]