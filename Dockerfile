# Builder stage (uses a multi-platform base image)
FROM --platform=$TARGETPLATFORM ekidd/rust-musl-builder AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --target=$TARGETPLATFORM --target-dir build

# Production stage
FROM scratch
COPY --from=builder /app/build/release/renvsubst /
ENTRYPOINT ["/renvsubst"]
