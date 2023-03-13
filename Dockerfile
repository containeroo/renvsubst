FROM scratch
ARG TARGET
COPY ./build/target/${TARGET}/release/renvsubst /renvsubst
ENTRYPOINT ["/renvsubst"]
