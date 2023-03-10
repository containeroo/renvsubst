# Build stage
FROM rust:latest AS builder

ARG TARGETARCH
WORKDIR /app
COPY . .

RUN case $TARGETARCH in \
      "amd64") \
        ENV PLATFORM=x86_64-unknown-linux-gnu \
        ENV COMPILER= \
        ;; \
      "arm64") \
        ENV PLATFORM=aarch64-unknown-linux-gnu \
        ENV COMPILER=gcc-aarch64-linux-gnu \
        ;; \
      "arm") \
        ENV PLATFORM=armv7-unknown-linux-gnueabihf \
        ENV COMPILER=gcc-arm-linux-gnueabihf \
        ;; \
      *) echo "Unsupported platform: $TARGETARCH"; exit 1; ;; \
    esac

RUN apt-get update && apt-get install -y unzip $COMPILER
RUN rustup target add $PLATFORM
RUN cargo build --release --target $PLATFORM

# Final stage
FROM scratch
COPY --from=builder /app/target/$PLATFORM/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
