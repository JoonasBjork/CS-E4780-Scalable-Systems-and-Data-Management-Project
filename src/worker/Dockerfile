FROM rust:1.82-bullseye AS builder

RUN apt update
RUN apt install -y dnsutils

WORKDIR /app

# Build dependencies
COPY Cargo.toml Cargo.lock ./ 

COPY startup.sh /app/startup.sh
RUN chmod +x /app/startup.sh

# Optimization to create a dummy main file to precompile dependencies before bringing the real project
# https://github.com/rust-lang/cargo/issues/2644#issuecomment-2335499312
RUN \
  mkdir -v src && \
  echo 'fn main() {}' > src/main.rs && \
  cargo build --release && \
  rm -Rvf src

# Cargo.lock
COPY src/ ./src/
RUN \
  touch src/main.rs && \
  cargo build --release

# The binary is in /app/target/release/worker

CMD ["sh", "/app/startup.sh"]
