FROM --platform=linux/amd64 rust:1-buster as builder-arm64

RUN apt update && apt upgrade -y
RUN apt install -y g++-arm-linux-gnueabihf libc6-dev-armhf-cross pkg-config libssl-dev

ENV RUST_TARGET=armv7-unknown-linux-gnueabihf

RUN rustup target add armv7-unknown-linux-gnueabihf
RUN rustup toolchain install stable-armv7-unknown-linux-gnueabihf

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
    CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++



FROM --platform=linux/amd64 rust:1-buster as builder-amd64

RUN apt update && apt upgrade -y
RUN apt install -y llvm-dev libclang-dev clang pkg-config libssl-dev

ENV RUST_TARGET=x86_64-unknown-linux-gnu



FROM --platform=linux/amd64 builder-${TARGETARCH} as builder

WORKDIR /code

COPY . .

RUN cargo build --release --target ${RUST_TARGET} --all-features

RUN cp /code/target/${RUST_TARGET}/release/controller /dmtrop
# TODO: switch into a command
RUN cp /code/target/${RUST_TARGET}/release/crds /dmtrcrds

FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /dmtrop /usr/local/bin/dmtrop
# TODO: switch into a command
COPY --from=builder /dmtrcrds /usr/local/bin/dmtrcrds

ENTRYPOINT [ "dolos" ]
