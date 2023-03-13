# Builder stage (uses a multi-platform base image)
FROM --platform=$TARGETPLATFORM rust:1.68.0-slim-buster AS builder
RUN apt-get update && apt-get install -y build-essential
WORKDIR /app
COPY . .
RUN cargo build --release --target-dir build
RUN /bin/bash -c ' \
    STRIP=""; \
    case $TARGETPLATFORM in \
      linux/amd64) STRIP="strip" ;; \
      linux/arm/v7) STRIP="arm-linux-gnueabihf-strip" ;; \
      linux/arm64) STRIP="aarch64-linux-gnu-strip" ;; \
      *-pc-windows-msvc) STRIP="" ;; \
    esac; \
    if [ -n "${STRIP}" ]; then \
      ${STRIP} "/app/build/release/renvsubst"; \
    fi'

# Production stage
FROM scratch
COPY --from=builder /app/build/release/renvsubst /
ENTRYPOINT ["/renvsubst"]
