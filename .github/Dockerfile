FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY binaries/Linux-GNU-${TARGETARCH}-dmtrd /bin/dmtrd
RUN chmod +x /bin/dmtrd
RUN ln -s /bin/dmtrd /dmtrd

ENTRYPOINT ["dmtrd"]