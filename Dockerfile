FROM alpine:latest as builder

RUN case $(uname -m) in \
    x86_64) TARGET=x86_64-unknown-linux-musl ;; \
    aarch64) TARGET=aarch64-unknown-linux-musl ;; \
    *) echo "Unsupported architecture $(uname -m)" >&2; exit 1 ;; \
    esac \
    mkdir -p /app \
    cp ./target/${TARGET}/release/renvsubst /app/

FROM scratch
COPY --from=builder /app/renvsubst /
ENTRYPOINT ["./renvsubst"]
