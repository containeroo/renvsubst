# Use a base image that supports multiple architectures
FROM --platform=${BUILDPLATFORM} rust:latest AS builder

# Set up the build environment
WORKDIR /app
COPY . .

# Install cross and build the application for the current target
RUN cargo install cross
RUN cross build --target=${TARGETPLATFORM}

# Create the final image
FROM scratch
COPY --from=builder /app/target/${TARGETPLATFORM}/release/renvsubst .
ENTRYPOINT ["./renvsubst"]
