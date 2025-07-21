FROM rust:1.88.0-alpine AS builder

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
COPY . .

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache \
    libgcc \
    musl \
    openssl \
    ca-certificates

COPY --from=builder /usr/src/app/target/release/subindexer /usr/local/bin/subindexer

ENV RUST_LOG=info
ENV PORT=8080

EXPOSE 8080

CMD ["subindexer"] 