FROM scratch
ARG TARGET
COPY ./renvsubst-v${TARGET}/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
