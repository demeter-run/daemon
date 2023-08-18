FROM alpine:latest

ARG TARGETARCH
COPY dmtrop-${TARGETARCH} /bin/dmtrop
RUN ln -s /bin/dmtrop /dmtrop

ENTRYPOINT ["dmtrop"]