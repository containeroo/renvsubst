# Build stage
FROM rust:latest AS builder

WORKDIR /app

COPY . .

ARG TARGETPLATFORM
RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        export TARGET=x86_64-unknown-linux-musl; \
    else \
        export TARGET=armv7-unknown-linux-musleabihf; \
    fi && \
    rustup target add $TARGET && \
    cargo install --target $TARGET --path .

ENV TARGET $TARGET

# Final stage
FROM scratch
COPY --from=builder /app/target/${TARGET}/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
