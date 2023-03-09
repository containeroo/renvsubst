# Build stage
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install the musl target
RUN rustup target add $TARGETPLATFORM

# Install the musl toolchain
RUN apt-get update && \
    apt-get install -y musl-tools

WORKDIR /usr/src/app
COPY . .

# Build the application for multiple architectures using cross and musl
RUN CROSS_TARGET=$TARGETPLATFORM \
    CROSS_RUSTFLAGS="--target=$TARGETPLATFORM -C linker=musl-gcc" \
    cross build --release

# Final stage
FROM scratch
ARG RUST_BINARY
COPY --from=builder /usr/src/app/target/$TARGETPLATFORM/release/$RUST_BINARY .
ENTRYPOINT [ "./$RUST_BINARY" ]
