# Build stage
FROM rust:latest AS builder

WORKDIR /app
COPY . .

# Build the application for x86_64
RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --target x86_64-unknown-linux-musl --release

# Build the application for armv7
RUN rustup target add armv7-unknown-linux-musleabihf && \
    cargo build --target armv7-unknown-linux-musleabihf --release

# Final stage for x86_64
FROM scratch

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/renvsubst /renvsubst

ENTRYPOINT ["/app"]

# Final stage for armv7
FROM scratch

COPY --from=builder /usr/src/app/target/armv7-unknown-linux-musleabihf/release/renvsubst /renvsubst

ENTRYPOINT ["/renvsubst"]
