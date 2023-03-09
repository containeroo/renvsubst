FROM busybox:latest as builder
ARG BIN_PATH

RUN echo "bin path: ${BIN_PATH}" && ls -lah ${BIN_PATH}
RUN mkdir ./tmp2
COPY ${BIN_PATH} ./tmp2/
RUN ls -lah ./tmp2/

FROM scratch
ARG BIN_PATH
COPY --from=builder ${BIN_PATH} /
ENTRYPOINT ["./renvsubst"]
