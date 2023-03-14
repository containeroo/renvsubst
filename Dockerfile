FROM scratch
ARG TARGET
ARG VERSION
COPY ./renvsubst-v${VERSION}-${TARGET}/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
