# SpiderMonkey

Three JavaScript workloads run on SpiderMonkey compiled to Wasm. `runtime.cpp` embeds the
engine, evaluates the benchmark's JS from `js/<name>/`, then calls `main(input)` — and only
that call sits between `bench_start()` and `bench_end()`. The `<name>.input` file is read
before the measured region and passed to `main` as a string.

The JS sources are baked into the Wasm at build time (`build.sh` runs them through `xxd` into
`js_files.h`), so changing anything under `js/` requires a rebuild. This directory is in
`build-all.sh`'s `SKIPLIST` because that rebuild — which builds SpiderMonkey from source — is
known to fail, so in practice the `*.input` files are the only tunable knob here.

## Instruction counts

`spidermonkey-regex` (~107M) and `spidermonkey-json` (~111M) sit in the ~100M band described in
[../README.md].

**`spidermonkey-markdown` cannot reach that band by resizing its input.** `marked.parse`
compiles its inline-lexer regexes lazily on first use, and that one-time cost lands inside the
measured region. Measured floor:

| `spidermonkey-markdown.input` | instructions |
| --- | --- |
| `---\n` (4 B, a thematic break — no inline content, so no regex compilation) | 3.4M |
| `hi\n` (3 B) | 247.6M |
| 162 B | 252.4M |
| 3,069 B (current) | 288.6M |
| 22,620 B (the full CommonMark spec preamble, previously) | 852.8M |

So any input with even one word of inline text costs ~248M, and the marginal cost is only
~29k instructions/byte. The input is set to the first 3,069 bytes of the CommonMark spec —
still real markdown (frontmatter, headings, links, inline code, prose paragraphs) and the
closest practical approach to 100M, but ~2.9x over target.

Moving the regex compilation out of the measured region would mean adding a warm-up
`marked.parse(...)` at the top level of `js/markdown/main.js` (top-level JS is evaluated before
`bench_start()`). That is the fix if this directory ever becomes rebuildable.
