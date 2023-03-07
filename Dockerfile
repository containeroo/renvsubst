# Base image
FROM rust:1.55-slim-buster AS builder

# Set up build environment
WORKDIR /renvsubst
RUN apt-get update && apt-get install -y build-essential

# Build dependencies
COPY ./ .
RUN cargo build --release

# Build final image
FROM scratch
COPY --from=builder /renvsubst/target/release/renvsubst /renvsubst
CMD ["/renvsubst"]
