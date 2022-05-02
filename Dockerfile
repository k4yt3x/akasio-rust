FROM rust:1.59-alpine3.15 as builder
COPY . /app
WORKDIR /app
RUN apk add --no-cache --virtual .build-deps \
        make \
        musl-dev \
        openssl-dev \
        perl \
        pkgconfig \
    && cargo build --release --target x86_64-unknown-linux-musl

FROM gcr.io/distroless/static:nonroot
LABEL maintainer="K4YT3X <i@k4yt3x.com>" \
      org.opencontainers.image.source="https://github.com/k4yt3x/akasio-rust" \
      org.opencontainers.image.description="A simple Rust program that redirects HTTP requests"
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/akasio \
                    /usr/local/bin/akasio
USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/akasio"]
