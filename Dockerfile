FROM rust:1.53.0 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM scratch
COPY --from=builder /app/target/release/renvsubst /renvsubst

ENTRYPOINT ["/renvsubst"]
