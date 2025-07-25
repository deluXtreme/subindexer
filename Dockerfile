FROM rust:1.88.0-slim-bullseye AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/subindexer /usr/local/bin/subindexer

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["subindexer"] 