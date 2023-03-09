# Build stage
FROM --platform=$BUILDPLATFORM rust:latest AS builder

# Install the musl toolchain
RUN apt-get update && \
    apt-get install -y musl-tools

WORKDIR /usr/src/app
COPY . .

# Build the application for multiple architectures using cross and musl
ARG TARGETPLATFORM
RUN rustup target add ${TARGETPLATFORM} && \
    cross build --target ${TARGETPLATFORM} --release

# Final stage
FROM scratch
ARG RUST_BINARY
COPY --from=builder /usr/src/app/target/${TARGETPLATFORM}/release/$RUST_BINARY .
ENTRYPOINT [ "./$RUST_BINARY" ]
