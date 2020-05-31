Building from source
====================

Building from source is rather easy, a [rust toolchain](https://www.rust-lang.org/tools/install) newer than `v1.41.0` is required.

For a debug build (faster to compile), run `cargo build`.

For a release build, run `cargo build --release --locked`.

Once compiled, the executable is located at `target/release/peertube-viewer-rs`. Completion for a variety of shells is also available in `completions/`.
