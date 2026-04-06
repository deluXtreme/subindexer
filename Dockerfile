FROM rust:1.88.0-slim-bullseye AS builder

WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates libsqlite3-0 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/subindexer /usr/local/bin/subindexer

# Copy seed database for initialization
COPY arak.db /opt/seed/arak.db

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["subindexer"] 
