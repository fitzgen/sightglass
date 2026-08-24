# BLAKE3

This benchmark is similar to [../blake3-scalar], but the build compiles BLAKE3's hand-written
SSE2 implementation (`blake3_sse2.c`): `wasm_sse_compat.h` maps its x86 SSE2 intrinsics onto
Wasm SIMD (via `<wasm_simd128.h>`), and a small patch forces BLAKE3's runtime dispatcher to
select the SSE2 kernels on wasm.

Both benchmarks hash whatever is in their own `default.input`, and both are sized to execute
~100M Wasm instructions (see [../README.md]). Because the SIMD kernels do more work per
instruction, this benchmark needs a larger input than [../blake3-scalar] to hit that target, so
the two `default.input` files — and therefore the two expected hashes — differ. Feeding both
benchmarks the same input does still produce the same hash; that is how
`benchmark.stderr.expected` here was cross-checked.
