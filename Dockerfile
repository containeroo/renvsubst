# Build stage
FROM rust:latest AS builder

ARG TARGETARCH
WORKDIR /app
COPY . .

RUN case $TARGETARCH in \
      "amd64") export PLATFORM="x86_64-unknown-linux-gnu"; export COMPILER=""; ;; \
      "arm64") export PLATFORM="aarch64-unknown-linux-gnu"; export COMPILER="gcc-aarch64-linux-gnu"; ;; \
      "arm") export PLATFORM="armv7-unknown-linux-gnueabihf"; export COMPILER="gcc-arm-linux-gnueabihf"; ;; \
      *) echo "Unsupported platform: $TARGETARCH"; exit 1; ;; \
    esac

RUN apt-get update && apt-get install -y unzip $COMPILER
RUN rustup target add $PLATFORM
RUN cargo install --target $PLATFORM --target-dir build


FROM scratch
COPY --from=builder /app/build/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
