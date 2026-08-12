# Image Classification Wasmtime Benchmark

A benchmark that runs an image classifier in pure Wasm. This can be used to
benchmark the performance of float heavy computations.

Note that the classifier model is not included in the repo because it is large
and is instead downloaded if needed when running the `setup.sh` script.

## Instruction count

This benchmark executes ~6.6G Wasm instructions, far above the ~100M that the
rest of the corpus targets (see `benchmarks/README.md`). It is the most expensive
benchmark in the tree.

The measured region is one forward pass of MobileNetV2, plus decoding and
resizing `input.png`. Shrinking `input.png` to the model's native 224x224 (so
that decode and resize become negligible) only brings it down to ~5.45G, so the
inference itself accounts for ~83% of the cost. MobileNetV2's input shape is
fixed at 224x224 by `assets/mobilenetv2-7.onnx`, so there is no workload knob
short of swapping the model for a smaller one, which would change what this
benchmark measures.

It is therefore left out of band deliberately.
