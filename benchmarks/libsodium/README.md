# libsodium

The [libsodium](https://libsodium.org/) test suite, built as one Wasm benchmark per
upstream test in `test/default/`. `add-bench-calls.diff` patches upstream's
`test/default/cmptest.h` so that the real `main()` wraps the test body (which upstream
renames to `xmain()`) in `bench_start()`/`bench_end()`.

## Upstream pin

Because the set of benchmarks here is derived from the set of files in upstream's
`test/default/`, upstream renaming or deleting a test silently changes which `.wasm` files
this directory produces — and leaves the old one behind as an untunable orphan. The
`Dockerfile` therefore pins `LIBSODIUM_COMMIT` to a commit hash rather than to a tag or a
release tarball, either of which upstream can replace in place. Bumping that hash is the
only thing that can change the benchmark set; when you do bump it, diff the built `.wasm`
names against the checked-in ones and add or remove `.input` files and `../*.suite` entries
to match.

## Tuning

The only knob is how many times each test body runs. `main()` reads it from a sibling
input file named after the test:

```c
iterations = (unsigned int) bench_read_long("./libsodium-" TEST_NAME ".input", ITERATIONS);
bench_start();
for (i = 0; i < iterations; i++) {
    if (xmain() != 0) { abort(); }
}
bench_end();
```

`TEST_NAME` is already defined by every upstream test file before it includes
`cmptest.h`, and it always matches the file stem, so `libsodium-auth7.wasm` reads
`./libsodium-auth7.input`. The read happens before `bench_start()`, so it is not
measured, and it falls back to the compiled-in `ITERATIONS` (which the `Dockerfile`
sets to 1) when the file is absent.

This means **retuning any of these benchmarks needs no Docker rebuild** — just edit the
input file. `default.stdout.expected` is a single `0` and does not depend on the
iteration count.

There is deliberately no per-test surgery inside the `xmain()` bodies: they are upstream
code, and keeping them untouched is what makes the patch small enough to rebase onto a
new libsodium release.

## Status

45 of the 78 tests are within the 80M-120M band described in [../README.md](../README.md).
The rest cannot get there with an iteration count as the only lever.

### Over the band even at one iteration (27)

One iteration of the test body already costs more than 120M instructions, so there is no
integer iteration count that lands in the band.

| test | instructions | test | instructions |
| --- | --- | --- | --- |
| `core_ristretto255` | 39.01G | `verify1` | 852.6M |
| `core_ed25519` | 35.34G | `metamorphic` | 442.5M |
| `box8` | 28.94G | `auth5` | 432.4M |
| `pwhash_scrypt` | 21.23G | `secretbox8` | 423.0M |
| `sign` | 15.20G | `box_easy2` | 384.2M |
| `box7` | 9.70G | `xchacha20` | 316.8M |
| `pwhash_argon2id` | 5.92G | `scalarmult8` | 292.9M |
| `pwhash_argon2i` | 5.20G | `sodium_utils` | 242.9M |
| `ed25519_convert` | 4.17G | `auth7` | 228.2M |
| `core3` | 1.64G | `aead_aegis256` | 151.1M |
| `pwhash_scrypt_ll` | 1.40G | `secretbox7` | 140.3M |
| `secretbox_easy2` | 1.18G | `aead_aegis128l` | 136.1M |
| `stream` | 1.12G | `onetimeauth7` | 131.6M |
| `stream2` | 1.03G |  |  |

### No measurable work on Wasm (6)

These bodies are empty or near-empty once compiled to Wasm, so no iteration count can
reach the band. They are left at 1 iteration rather than padded with a loop that would
measure nothing but loop overhead.

| test | instructions | why |
| --- | --- | --- |
| `aead_aes256gcm` | 65 | Guarded by `crypto_aead_aes256gcm_is_available()`, which is false on Wasm. |
| `aead_aes256gcm2` | 27 | Same AES-NI availability check as `aead_aes256gcm`. |
| `misuse` | 0 | Its body is `#ifdef HAVE_CATCHABLE_ABRT`, and `build.zig` only defines that for Linux and macOS, so on Wasm the test is just `return 0`. |
| `onetimeauth2` | 0 | Its only statement is `printf("%d\n", crypto_onetimeauth_verify(...))`, and `cmptest.h` defines `printf(...)` to `do { } while(0)` — so the argument is never evaluated and the verify never runs. |
| `sodium_core` | 236 | Only runtime CPU feature detection, all false on Wasm. |
| `sodium_version` | 25 | Only version-string getters. |

### Not deterministic (7)

These seed libsodium's RNG from the OS (WASI `random_get`) and then size their work from
it, so their instruction count varies between runs of the *same* Wasm. They cannot be
tuned to a target at all.

| test | observed range | note |
| --- | --- | --- |
| `box_easy2` | 221M - 3.79G | Cost is quadratic in a random message length. |
| `secretbox_easy2` | 261M - 9.02G | Cost is quadratic in a random message length. |
| `sodium_utils` | 243M - 377M | Over the band at any iteration count regardless. |
| `aead_aegis256` | 151M - 153M | Over the band at any iteration count regardless. |
| `aead_aegis128l` | 134M - 137M | Over the band at any iteration count regardless. |
| `codecs` | 97M - 98M | In band; the jitter is ~1%, so it is left alone. |
| `secretstream_xchacha20poly1305` | 98M - 102M | In band; the jitter is ~4%, so it is left alone. |

Because of this, the single figures quoted for `box_easy2`, `secretbox_easy2`,
`sodium_utils`, `aead_aegis256`, and `aead_aegis128l` in the table above are just whatever
one run happened to produce.

This conflicts with the determinism requirement in [../README.md](../README.md); it is a
pre-existing property of these upstream tests rather than something the iteration knob
introduced.

## License

libsodium is licensed under the ISC license.
