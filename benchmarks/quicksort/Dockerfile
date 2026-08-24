# This two-phase Dockerfile allows us to avoid re-downloading APT packages and wasi-sdk with every
# build.

ARG WASI_SDK_VERSION=28

# First, retrieve wasi-sdk:

FROM ubuntu:24.04 AS builder
ARG WASI_SDK_VERSION
WORKDIR /
RUN apt update && apt install -y wget

# Download and extract wasi-sdk for the host architecture, then normalize the install path to
# `/wasi-sdk` so that the stage below does not have to know the version or architecture.
RUN ARCH=$(uname -m); \
    case "$ARCH" in \
    x86_64) WASI_ARCH=x86_64 ;; \
    aarch64 | arm64) WASI_ARCH=arm64 ;; \
    *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; \
    esac; \
    TARBALL=wasi-sdk-${WASI_SDK_VERSION}.0-${WASI_ARCH}-linux.tar.gz; \
    wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_VERSION}/$TARBALL; \
    tar xvf $TARBALL; \
    mv wasi-sdk-${WASI_SDK_VERSION}.0-${WASI_ARCH}-linux /wasi-sdk

# Second, compile the benchmark to Wasm.

FROM ubuntu:24.04
WORKDIR /
COPY --from=builder /wasi-sdk /wasi-sdk/

# Set common env vars.
ENV CC=/wasi-sdk/bin/clang
ENV CXX=/wasi-sdk/bin/clang++
ENV LD=/wasi-sdk/bin/lld
ENV CFLAGS=--sysroot=/wasi-sdk/share/wasi-sysroot
ENV CXXFLAGS=--sysroot=/wasi-sdk/share/wasi-sysroot
ENV PATH /wasi-sdk

# Compile `benchmark.c` to `./benchmark.wasm`.
COPY quicksort.c .
COPY sightglass.h .
WORKDIR /benchmark
RUN $CC $CFLAGS ../quicksort.c -O3 -g -I.. -o benchmark.wasm
# We output the Wasm file to the `/benchmark` directory, where the client
# expects it.
