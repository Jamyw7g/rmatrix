# RMatrix

Rmatrix is similar to the [cmatrix](https://github.com/abishekvashok/cmatrix), but it wrote it in `Rust`. Get up to 75% performance improvement compared to cmatrix.

## Build and Install

**Minimum Rust requires 1.53.0**

```shell
# build
cargo build --release

# install
cargo install --path .
```

## Performance (2026-04-06)

This round focused on render-path and matrix-update hot loops:

- Switched from per-character `print!` to per-frame buffered output.
- Removed hot-path string clones.
- Replaced `Stylize` formatting in loops with ANSI color state switching.
- Added minimum poll delay to avoid busy-spin when speed is `0`.
- Compacted `Item` storage (`u8 + flags`).
- Reduced index arithmetic in `Matrix::next()`.
- Added RNG pooling/bit-pool usage in hot loops.

### Benchmark summary (local machine)

Measured with a temporary `perf_compare` binary on the same workspace, 3 runs each.

1. `Matrix::next()` (10,000 frames, 220x70):
   - Before: 56.504 ms (median)
   - After: 48.624 ms (median)
   - Improvement: **~13.94%**
2. Render string build (old/new logic simulation, rainbow on/off):
   - Before: 178.654 ms (median)
   - After: 56.810 ms (median)
   - Improvement: **~68.20%**

Note: render benchmark isolates frame-string construction and does not include real terminal I/O latency.

## Screencasts

![rmatrix](./gifs/rmatrix.gif)
