FROM busybox:latest
ARG BIN_PATH

RUN echo "bin path: ${BIN_PATH}" && ls -lah ${BIN_PATH}

FROM scratch
ARG BIN_PATH
COPY ${BIN_PATH} .
ENTRYPOINT ["./renvsubst"]
