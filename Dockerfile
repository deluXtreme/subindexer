FROM rust:1.87.0-slim-bullseye AS builder

RUN apk add --no-cache \
    musl-dev \
    build-base \
    openssl-dev \
    pkgconfig \
    cmake \
    perl \
    bash \
    curl \
    linux-headers

WORKDIR /usr/src/app
# 1. Copy only manifests first and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# 2. Copy the rest of your source code
COPY . .

# 3. Build the application
RUN cargo build --release

# 4. Remove development dependencies
RUN cargo clean

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/subindexer /usr/local/bin/subindexer

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["subindexer"] 