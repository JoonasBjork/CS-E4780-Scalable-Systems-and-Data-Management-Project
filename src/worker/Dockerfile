FROM rust:1.82-bullseye AS builder

WORKDIR /app

# Build dependencies
COPY Cargo.toml Cargo.lock ./ 

# Optimization to create a dummy main file to precompile dependencies before bringing the real project
# https://github.com/rust-lang/cargo/issues/2644#issuecomment-2335499312
RUN \
  mkdir -v src && \
  echo 'fn main() {}' > src/main.rs && \
  cargo build && \
  rm -Rvf src

# Cargo.lock
COPY src/ ./src/
RUN \
  touch src/main.rs && \
  cargo build


# Use smaller base image for final build
FROM debian:bullseye-slim
COPY --from=builder /app/target/debug/worker /usr/local/bin/worker


#### This portion is a hack to get the container identifier for selecting streams. 
RUN apt update
RUN apt install -y dnsutils

COPY startup.sh /usr/local/bin/startup.sh
RUN chmod +x /usr/local/bin/startup.sh

CMD ["sh", "/usr/local/bin/startup.sh"]



# RUN cargo build --release

# Build app
# RUN cargo build --release
# RUN cargo install --path .

# COPY --from=builder /app/target/release/worker /usr/local/bin/worker
