# Base image
FROM rust:slim-buster AS builder

# Set up build environment
WORKDIR /renvsubst
RUN apt-get update && apt-get install -y build-essential

# Build dependencies
COPY ./ .
# workarount for arm builds
RUN --mount=type=tmpfs,target=/.cargo CARGO_HOME=/.cargo cargo build --release

# Build final image
FROM scratch
COPY --from=builder /renvsubst/target/release/renvsubst ./renvsubst
RUN ls
ENTRYPOINT ["./renvsubst"]
