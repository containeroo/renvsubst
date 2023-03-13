FROM scratch
ARG BIN_PATH
COPY ${BIN_PATH} /
ENTRYPOINT ["./renvsubst"]
