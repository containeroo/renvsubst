# Builder stage (uses a multi-platform base image)
FROM --platform=$TARGETPLATFORM rust:1.57-slim-bullseye AS builder
RUN apt-get update && apt-get install -y build-essential
WORKDIR /app
COPY . .
RUN cargo build --release --target-dir build

# Production stage
FROM scratch
COPY --from=builder /app/build/release/renvsubst /
ENTRYPOINT ["/renvsubst"]
