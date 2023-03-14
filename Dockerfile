FROM rust:1.68.0-slim-buster AS builder

WORKDIR /app

COPY . .

# Set the target platform based on the machine architecture
RUN case "$(uname -m)" in \
      x86_64) TARGET=x86_64-unknown-linux-musl ;; \
      aarch64) TARGET=aarch64-unknown-linux-musl ;; \
      armv7l) TARGET=armv7-unknown-linux-musleabihf ;; \
      *) echo "Unsupported architecture: $(uname -m)" && exit 1 ;; \
    esac && \
    rustup target add $TARGET && \
    cargo build --target $TARGET --release --target-dir build

# Create a new stage for the final image
FROM scratch
COPY --from=builder /app/build/*/release/renvsubst .
CMD ["./renvsubst"]
