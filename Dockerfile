# Build stage
FROM rust:latest AS builder

WORKDIR /app

COPY . .

ARG TARGET

RUN rustup target add ${TARGET}
RUN cargo install --target ${TARGET} --path .

# Final stage
FROM scratch
ARG TARGET
COPY --from=builder /app/target/${TARGET}/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
