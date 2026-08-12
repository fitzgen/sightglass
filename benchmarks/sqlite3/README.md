# sqlite3

A WebAssembly build of SQLite's official `speedtest1.c` benchmark program,
ported from [JetStream 3](https://github.com/WebKit/JetStream/tree/main/sqlite3)
for use with Sightglass.

Quoting from [its description](https://sqlite.org/cpu.html):

> This program strives to exercise the SQLite library in a way that is typical
> of real-world applications. Of course, every application is different, and so
> no test program can exactly mirror the behavior of all applications.

Since SQLite is a very widely used database and provides an official and popular
upstream WebAssembly port, this is a realistic, larger WebAssembly program.

Originally built from SQLite 3.48.0 with Emscripten SDK 3.1.73.

## Porting notes

The original JetStream 3 module was compiled by Emscripten and paired with a
JavaScript driver. Since sightglass does not use JavaScript, the module was
modified to:

- Remove Emscripten `"env"` imports and replace them with stubs (except
  `emscripten_resize_heap`, which delegates to `memory.grow`).
- Add `"bench" "start"` and `"bench" "end"` imports for Sightglass timing.
- Add a `"_start"` export that calls `__wasm_call_ctors`, `bench.start`,
  `wasm_main`, and `bench.end`.
- Execute the sqlite3 benchmarks with `szTest=1` instead of `szTest=100`.

## Instruction count

This benchmark executes ~459M Wasm instructions, which is above the ~100M that
the rest of the corpus targets (see `benchmarks/README.md`).

`szTest` is the only knob available: the C sources are not in this repository,
so the size is patched directly in `sqlite3.wat` as the constant
`i64.const 4294967297` (`= 2^32 + szTest`, sharing a 64-bit store with an
adjacent field). It is already at its minimum of `1`, which brought the
benchmark down from ~17.1G instructions at `szTest=25`.

Going lower would require restricting which of speedtest1's test cases run,
which would change what the benchmark measures, so this is left out of band
deliberately. It is still well above `scripts/pca.R`'s
`MIN_DYNAMIC_INST_COUNT`, so it continues to participate in the PCA.

## License

The SQLite source code is public domain:

> The author disclaims copyright to this source code. In place of a legal
> notice, here is a blessing:
>
> - May you do good and not evil.
> - May you find forgiveness for yourself and forgive others.
> - May you share freely, never taking more than you give.
