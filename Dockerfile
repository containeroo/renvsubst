FROM alpine:latest

RUN ls -lh ./build/target/${TARGET}/release

FROM scratch
ARG TARGET
COPY ./build/target/${TARGET}/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
