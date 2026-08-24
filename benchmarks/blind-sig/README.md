# `blind-sig`

This benchmark computes RSA Blind Signatures using [the `blind-rsa-signatures`
crate](https://crates.io/crates/blind-rsa-signatures). Generating these
signatures is a useful test of math on big integers.

The `blind-rsa-signatures` crate is licensed under the MIT license.

## Instruction count

This benchmark executes ~245M Wasm instructions, which is above the ~100M that
the rest of the corpus targets (see `benchmarks/README.md`).

The measured region is a single blind signature — blind, blind-sign, finalize —
so the only knob is the modulus size baked into `secret.der`, and private-key
RSA work scales roughly cubically with it. `secret.der` is already at the
smallest size the crate accepts: `blind-rsa-signatures` rejects any modulus
outside 2048–4096 bits, and 2048 bits is what brought this benchmark down from
~1.53G instructions at the original 4096 bits.

Getting to ~100M would need a non-standard sub-2048-bit modulus, which the crate
will not load, so this is left out of band deliberately. It is still well above
`scripts/pca.R`'s `MIN_DYNAMIC_INST_COUNT`, so it continues to participate in
the PCA.

`benchmark.stdout.expected` is the signature itself, so it must be regenerated
whenever `secret.der` changes.
