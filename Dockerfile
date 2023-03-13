FROM scratch
ARG TARGET
COPY ./build/${TARGET}/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
