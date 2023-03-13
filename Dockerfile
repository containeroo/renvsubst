FROM alpine:latest
ARG TARGET

WORKDIR ./build
RUN ls -lah .
RUN ls -lh ./${TARGET}/release

FROM scratch
ARG TARGET
COPY ./build/${TARGET}/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
