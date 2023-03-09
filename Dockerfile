# Build stage
FROM rust:latest AS builder

WORKDIR /app

COPY . .

RUN cargo install --path . --target-dir ./build

# Final stage
FROM scratch
COPY --from=builder /app/build/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
