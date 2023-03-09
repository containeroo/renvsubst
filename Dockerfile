FROM busybox:latest as builder
ARG BIN_PATH

RUN mkdir ./tmp
RUN echo "bin path: ${BIN_PATH}" && ls -lah ${BIN_PATH}
COPY ${BIN_PATH} ./tmp/

FROM scratch
ARG BIN_PATH
COPY --from=builder ${BIN_PATH} /
ENTRYPOINT ["./renvsubst"]
