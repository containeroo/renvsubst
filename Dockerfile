FROM scratch
ARG TARGET
COPY --from=builder /app/target/release/${TARGET}/renvsubst /
ENTRYPOINT ["/renvsubst"]
