# This Dockerfile builds all of libsodium's benchmarks with Zig.
FROM ubuntu:22.04
RUN apt update && apt install -y wget xz-utils unzip git

# Download Zig and retrieve a pinned commit of libsodium that can be built by it.
ARG ZIG_VERSION=0.12.0
WORKDIR /
# Download Zig for the host architecture and normalize the install path to `/zig` so that the rest
# of this file does not have to know the version or architecture.
RUN ARCH=$(uname -m); \
    case "$ARCH" in \
    x86_64) ZIG_ARCH=x86_64 ;; \
    aarch64 | arm64) ZIG_ARCH=aarch64 ;; \
    *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; \
    esac; \
    TARBALL=zig-linux-${ZIG_ARCH}-${ZIG_VERSION}.tar.xz; \
    wget https://ziglang.org/download/${ZIG_VERSION}/$TARBALL; \
    tar --extract --verbose --file=$TARBALL; \
    mv zig-linux-${ZIG_ARCH}-${ZIG_VERSION} /zig
ENV PATH="${PATH}:/zig"
RUN echo "zig version = $(zig version)"
# Fetch libsodium at a pinned commit -- the `1.0.20-RELEASE` tag as of this writing. Pin by commit
# hash rather than by tag or release tarball, both of which upstream can replace in place. This
# directory produces one benchmark per file in `test/default/`, so a test that upstream renames or
# deletes would otherwise silently change which `.wasm` files this build emits; with a hash, that
# can only happen when we deliberately bump `LIBSODIUM_COMMIT` here.
ARG LIBSODIUM_COMMIT=9511c982fb1d046470a8b42aa36556cdb7da15de
RUN git init /libsodium \
    && git -C /libsodium remote add origin https://github.com/jedisct1/libsodium \
    && git -C /libsodium fetch --depth 1 origin ${LIBSODIUM_COMMIT} \
    && git -C /libsodium checkout --detach ${LIBSODIUM_COMMIT}

# Build the benchmarks. Note how we patch the self-timing code with calls to the
# Sightglass `bench_start()` and `bench_end()` functions. Also note that each
# benchmark contains internal iterations; see `benchmarks_iterations` in
# `build.zig` and `ITERATIONS` in `cmptest.h`.
WORKDIR /libsodium
COPY sightglass.h src/libsodium/include/
COPY add-bench-calls.diff .
RUN git apply add-bench-calls.diff
RUN zig build -Dtarget=wasm32-wasi \
    -Doptimize=ReleaseFast -Dcpu=generic+bulk_memory+simd128 \
    -Denable_benchmarks=true -Diterations=1

# Place all of the WebAssembly benchmarks in `/benchmark`.
WORKDIR /benchmark
RUN cp /libsodium/zig-out/bin/*.wasm .
RUN for f in *.wasm; do mv "$f" "libsodium-$f"; done

# These benchmarks lose the `bench_end` import for some reason.
RUN rm libsodium-sodium_utils2.wasm libsodium-sodium_utils3.wasm

# These tests do no measurable work once compiled to Wasm -- their bodies are guarded by
# host-feature checks that are false on Wasm, or are empty after preprocessing -- so they
# measure nothing but harness overhead no matter how many iterations they run.
RUN rm libsodium-aead_aes256gcm.wasm libsodium-aead_aes256gcm2.wasm \
       libsodium-misuse.wasm libsodium-onetimeauth2.wasm \
       libsodium-sodium_core.wasm libsodium-sodium_version.wasm
