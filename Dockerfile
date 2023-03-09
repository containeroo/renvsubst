# Build stage
FROM rust:latest AS builder

WORKDIR /app

COPY . .

ARG TARGET

RUN rustup target add ${TARGET}
RUN cargo build --release --target ${TARGET}

# Final stage
FROM scratch
COPY --from=builder /app/target/${TARGET}/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
