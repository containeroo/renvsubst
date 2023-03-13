# Builder stage (uses a multi-platform base image)
FROM --platform=$TARGETPLATFORM ekidd/rust-musl-builder AS builder
WORKDIR /app
COPY . .
RUN TARGET=""; \
    case $TARGETPLATFORM in \
      linux/amd64) TARGET=" x86_64-unknown-linux-musl" ;; \
      linux/arm/v7) TARGET="arm-unknown-linux-musleabihf" ;; \
      linux/arm64) TARGET="arm-unknown-linux-musleabihf" ;; \
      *-pc-windows-msvc) TARGET="" ;; \
    esac; && \
    rustup target add $TARGET && \
    cargo build --release --target=$TARGETPLATFORM --target-dir build && \
    musl-strip /app/build/release/renvsubst

# Production stage
FROM scratch
COPY --from=builder /app/build/release/renvsubst /
ENTRYPOINT ["/renvsubst"]
