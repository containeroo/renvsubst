FROM scratch
COPY target/*/release/renvsubst ./
ENTRYPOINT ["./renvsubst"]
