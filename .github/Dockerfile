FROM alpine:latest

ARG TARGETARCH
COPY dmtrop-${TARGETARCH} /bin/dmtrop
RUN ln -s /bin/dmtrd /dmtrd

ENTRYPOINT ["dmtrd"]