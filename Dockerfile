FROM scratch
COPY renvsubst /renvsubst
ENTRYPOINT [ "/renvsubst" ]
