FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev

WORKDIR /renvsubst

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release

####################################################################################################
## Final image
####################################################################################################
FROM scratch

WORKDIR /renvsubst

# Copy our build
COPY --from=builder /renvsubst/target/x86_64-unknown-linux-musl/release/renvsubst ./

ENTRYPOINT ["./renvsubst"]
