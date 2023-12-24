FROM alpine:latest

ARG TARGETARCH
COPY dmtrd-${TARGETARCH} /bin/dmtrd
RUN ln -s /bin/dmtrd /dmtrd

ENTRYPOINT ["dmtrd"]