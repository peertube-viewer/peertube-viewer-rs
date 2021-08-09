Installation
============

**peertube-viewer-rs** is available on the AUR: 

- [peertube-viewer-rs](https://aur.archlinux.org/packages/peertube-viewer-rs/)
- [peertube-viewer-rs-bin](https://aur.archlinux.org/packages/peertube-viewer-rs-bin/)

Releases
--------

The download for the latest releases can be found on the download page of the project: [Download](https://peertube-viewer.com/download)

Releases can be found in the release page of the project: [releases](https://peertube-viewer.com/releases)

Building from source
--------------------

Building from source is rather easy, a [rust toolchain](https://www.rust-lang.org/tools/install) newer than `v1.53` is required.

For a debug build (faster to compile), run `cargo build`.

For a release build, run `cargo build --release --locked`.

Once compiled, the executable is located at `target/release/peertube-viewer-rs`.
Completion for a variety of shells is also available in `completions/`.
A manpage is available in the file `peertube-viewer-rs.1`.
