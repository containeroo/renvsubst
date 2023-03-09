# Build stage
FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# Final stage
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
